#![allow(non_snake_case)]
#![allow(unused_unsafe)]
use std::cell::RefCell;
use std::ptr::NonNull;

use icrate::ns_string;
use icrate::AppKit::{
    NSApp, NSApplication, NSApplicationActivationPolicyRegular, NSApplicationDelegate,
    NSApplicationMain, NSEventModifierFlagCommand, NSEventModifierFlagOption, NSMenu,
    NSMenuDelegate, NSMenuItem, NSWorkspace,
};
use icrate::Foundation::{
    MainThreadMarker, NSNotification, NSObject, NSObjectProtocol, NSString, NSURL,
};
use objc2::declare::{Ivar, IvarDrop};
use objc2::rc::{Allocated, Id};
use objc2::runtime::ProtocolObject;
use objc2::{declare_class, extern_methods, msg_send, mutability, sel, ClassType};

use xkcd_1975::{Action, ClickAction, Conditional, Data, Graph, MenuId, Reaction, State};

const NAME: &str = "XKCD 1975";

pub struct DelegateState {
    main_menus: Vec<(Conditional, Id<CustomMenu>)>,
    state: RefCell<State>,
    graph: Graph,
}

declare_class!(
    struct Delegate {
        state: IvarDrop<Box<DelegateState>, "_state">,
    }

    mod ivars;

    unsafe impl ClassType for Delegate {
        type Super = NSObject;
        type Mutability = mutability::MainThreadOnly;
        const NAME: &'static str = "Delegate";
    }

    unsafe impl Delegate {
        #[method(init)]
        fn init(this: &Self) -> Option<&mut Self> {
            let mtm = MainThreadMarker::new().unwrap();
            let this: Option<&mut Self> = unsafe { msg_send![super(this), init] };
            this.map(|this| {
                let data = Data::load();
                let main_menus = data
                    .root
                    .menu
                    .entries
                    .into_iter()
                    .map(|entry| {
                        let state = match &entry.reaction {
                            Reaction::SubMenu { submenu, on_hover } => MenuState {
                                id: submenu.id(&data.root.state),
                                on_hover: on_hover.clone(),
                            },
                            _ => unreachable!(),
                        };
                        let menu = CustomMenu::new(mtm, state);
                        unsafe { menu.setAutoenablesItems(false) };
                        unsafe { menu.setDelegate(Some(ProtocolObject::from_ref(this))) };

                        (entry.display, menu)
                    })
                    .collect();
                Ivar::write(
                    &mut this.state,
                    Box::new(DelegateState {
                        main_menus,
                        state: RefCell::new(data.root.state),
                        graph: data.graph,
                    }),
                );
                this
            })
        }
    }

    unsafe impl NSObjectProtocol for Delegate {}

    unsafe impl NSApplicationDelegate for Delegate {
        #[method(applicationDidFinishLaunching:)]
        unsafe fn applicationDidFinishLaunching(&self, _: &NSNotification) {
            let mtm = MainThreadMarker::new().unwrap();
            let app = unsafe { NSApp.unwrap() };

            for (_, menu) in &self.state.main_menus {
                self.menuNeedsUpdate(&menu);
                let system_item = NSMenuItem::initWithTitle_action_keyEquivalent(
                    mtm.alloc(),
                    ns_string!(NAME),
                    None,
                    ns_string!(""),
                );
                menu.insertItem_atIndex(&system_item, 0);
            }

            self.update_main_menu();

            // Activate manually, since we're not being launched as an application bundle
            app.activateIgnoringOtherApps(true);
        }

        // Required to prevent a warning regarding saved application state
        // https://sector7.computest.nl/post/2022-08-process-injection-breaking-all-macos-security-layers-with-a-single-vulnerability/
        #[method(applicationSupportsSecureRestorableState:)]
        fn applicationSupportsSecureRestorableState(&self, _: &NSApplication) -> bool {
            true
        }
    }

    unsafe impl NSMenuDelegate for Delegate {
        #[method(menuWillOpen:)]
        unsafe fn _menuWillOpen(&self, menu: &NSMenu) {
            let MenuState { id, on_hover } = get_menu_state(menu);
            let data = &self.state.graph[id];

            let mut state = self.state.state.borrow_mut();
            state.update(on_hover);

            let items = menu.itemArray();
            let mut iter = items.iter();

            // Hacky code for the main menu having the system menu
            if menu.numberOfItems() as usize != data.entries.len() {
                assert_eq!(menu.numberOfItems() as usize - 1, data.entries.len());
                let _ = iter.next().unwrap();
            }

            for (item, entry) in iter.zip(&data.entries) {
                item.setHidden(!entry.display.evaluate(&state));
                item.setEnabled(entry.active.evaluate(&state));
            }
        }

        #[method(menuDidClose:)]
        fn _menuDidClose(&self, menu: &NSMenu) {
            let MenuState { id, .. } = get_menu_state(menu);
            let data = &self.state.graph[id];

            self.state.state.borrow_mut().update(&data.on_leave);
        }

        #[method(menuNeedsUpdate:)]
        fn _menuNeedsUpdate(&self, menu: &NSMenu) {
            if unsafe { menu.numberOfItems() } > 0 {
                return;
            }
            let mtm = MainThreadMarker::new().unwrap();
            let MenuState { id, .. } = get_menu_state(menu);
            dbg!(id);
            let data = &self.state.graph[id];

            for entry in &data.entries {
                let title = NSString::from_str(&entry.label);
                let item = unsafe {
                    NSMenuItem::initWithTitle_action_keyEquivalent(
                        mtm.alloc(),
                        &title,
                        None,
                        ns_string!(""),
                    )
                };
                match &entry.reaction {
                    Reaction::SubMenu { on_hover, submenu } => unsafe {
                        let menu = CustomMenu::new(
                            mtm,
                            MenuState {
                                id: submenu.id(&self.state.state.borrow()),
                                on_hover: on_hover.clone(),
                            },
                        );
                        menu.setAutoenablesItems(false);
                        menu.setDelegate(Some(ProtocolObject::from_ref(self)));
                        item.setSubmenu(Some(&menu));
                    },
                    // Ignore contents; handled in `click:` instead
                    Reaction::ClickAction { .. } => unsafe {
                        item.setTarget(Some(&self));
                        item.setAction(Some(sel!(click:)));
                    },
                }
                unsafe { menu.addItem(&item) };
            }
        }
    }

    unsafe impl Delegate {
        #[method(click:)]
        unsafe fn click(&self, item: &NSMenuItem) {
            let menu = item.menu().unwrap();
            let MenuState { id, .. } = get_menu_state(&menu);

            match &self.state.graph[id].entries[menu.indexOfItem(item) as usize].reaction {
                Reaction::SubMenu { .. } => {
                    unreachable!("found submenu where clickaction was expected")
                }
                Reaction::ClickAction { on_action, act } => {
                    // Update state before doing the action
                    self.state.state.borrow_mut().update(&on_action);

                    if let Some(act) = act {
                        match act {
                            ClickAction::ColapseMenu => {
                                // Do nothing, as the menu will close automatically after an item was clicked

                                // Except we _do_ need to update the main menu...
                                self.update_main_menu();
                            }
                            ClickAction::Nav { url } | ClickAction::Download { url, .. } => unsafe {
                                let url = NSURL::initWithString(
                                    NSURL::alloc(),
                                    &NSString::from_str(&url),
                                )
                                .unwrap();
                                let workspace = NSWorkspace::sharedWorkspace();
                                // TODO: Download properly
                                workspace.openURL(&url);
                            },
                            ClickAction::JSCall { js_call } => {
                                println!("Unimplemented action: {js_call:?}");
                            }
                        }
                    }
                }
            }
        }
    }
);

impl Delegate {
    fn update_main_menu(&self) {
        eprintln!("update main menu");
        let app = unsafe { NSApp.unwrap() };

        let menu = {
            let state = self.state.state.borrow();
            self.state
                .main_menus
                .iter()
                .find_map(|(cond, menu)| cond.evaluate(&state).then_some(menu))
                .expect("could not find a main menu")
        };

        unsafe {
            menu.itemAtIndex(0)
                .unwrap()
                .setSubmenu(Some(&create_system_menu(app)))
        };

        unsafe { self.menuWillOpen(&menu) };

        unsafe { app.setMainMenu(Some(&menu)) };
    }
}

fn get_menu_state(menu: &NSMenu) -> &MenuState {
    assert!(menu.is_kind_of::<CustomMenu>());
    let menu: *const NSMenu = menu;
    let menu: *const CustomMenu = menu.cast();
    // SAFETY: Checked above that the menu is an instance of CustomMenu
    let menu: &CustomMenu = unsafe { &*menu };
    &**menu.state
}

extern_methods!(
    unsafe impl Delegate {
        #[method_id(new)]
        fn new(mtm: MainThreadMarker) -> Id<Self>;
    }
);

pub struct MenuState {
    id: MenuId,
    on_hover: Action,
}

declare_class!(
    struct CustomMenu {
        state: IvarDrop<Box<MenuState>, "_state">,
    }

    mod ivars_menu;

    unsafe impl ClassType for CustomMenu {
        type Super = NSMenu;
        type Mutability = mutability::MainThreadOnly;
        const NAME: &'static str = "CustomMenu";
    }
);

impl CustomMenu {
    fn new(mtm: MainThreadMarker, state: MenuState) -> Id<Self> {
        let this: Allocated<Self> = mtm.alloc().unwrap();
        let this: *mut Self = unsafe { std::mem::transmute(this) };
        let this: Option<&mut Self> = unsafe { msg_send![super(this), init] };
        let this = this.unwrap();
        Ivar::write(&mut this.state, Box::new(state));
        unsafe { Id::new(this).unwrap() }
    }
}

fn create_system_menu(app: &NSApplication) -> Id<NSMenu> {
    unsafe {
        let mtm = MainThreadMarker::new().unwrap();

        let name = ns_string!(NAME); // NSProcessInfo::processInfo().processName();
        let menu = NSMenu::initWithTitle(mtm.alloc(), &name);

        menu.addItemWithTitle_action_keyEquivalent(
            &NSString::from_str(&format!("About {name}")),
            Some(sel!(orderFrontStandardAboutPanel:)),
            ns_string!(""),
        );
        menu.addItem(&NSMenuItem::separatorItem());

        let services = NSMenu::initWithTitle(mtm.alloc(), ns_string!("Services"));
        let services_item = NSMenuItem::initWithTitle_action_keyEquivalent(
            mtm.alloc(),
            ns_string!("Services"),
            None,
            ns_string!(""),
        );
        services_item.setSubmenu(Some(&services));
        app.setServicesMenu(Some(&services));
        menu.addItem(&services_item);
        menu.addItem(&NSMenuItem::separatorItem());

        menu.addItemWithTitle_action_keyEquivalent(
            &NSString::from_str(&format!("Hide {name}")),
            Some(sel!(hide:)),
            ns_string!("h"),
        );
        let hide_others = NSMenuItem::initWithTitle_action_keyEquivalent(
            mtm.alloc(),
            ns_string!("Hide Others"),
            Some(sel!(hideOtherApplications:)),
            ns_string!("h"),
        );
        hide_others
            .setKeyEquivalentModifierMask(NSEventModifierFlagCommand | NSEventModifierFlagOption);
        menu.addItem(&hide_others);
        menu.addItemWithTitle_action_keyEquivalent(
            ns_string!("Show All"),
            Some(sel!(unhideAllApplications:)),
            ns_string!(""),
        );
        menu.addItem(&NSMenuItem::separatorItem());

        menu.addItemWithTitle_action_keyEquivalent(
            &NSString::from_str(&format!("Ouit {name}")),
            Some(sel!(terminate:)),
            ns_string!("q"),
        );

        menu
    }
}

fn main() {
    let mtm = MainThreadMarker::new().unwrap();
    let app = unsafe { NSApplication::sharedApplication() };
    unsafe { app.setActivationPolicy(NSApplicationActivationPolicyRegular) };
    let delegate = Delegate::new(mtm);
    unsafe { app.setDelegate(Some(ProtocolObject::from_ref(&*delegate))) };

    unsafe { NSApplicationMain(0, NonNull::new((&mut []).as_mut_ptr()).unwrap()) };
}
