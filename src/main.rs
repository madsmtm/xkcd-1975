#![allow(non_snake_case)]
#![allow(unused_unsafe)]
use std::cell::RefCell;
use std::ptr::NonNull;

use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2::{declare_class, msg_send_id, mutability, sel, ClassType, DeclaredClass};
use objc2_app_kit::{
    NSApplication, NSApplicationActivationPolicy, NSApplicationDelegate, NSApplicationMain,
    NSEventModifierFlags, NSMenu, NSMenuDelegate, NSMenuItem, NSWorkspace,
};
use objc2_foundation::{
    ns_string, MainThreadMarker, NSNotification, NSObject, NSObjectProtocol, NSString, NSURL,
};

use xkcd_1975::{Action, ClickAction, Conditional, Data, Graph, Reaction, State, SubMenu};

const NAME: &str = "XKCD 1975";

pub struct DelegateState {
    main_menus: Vec<(Conditional, Retained<CustomMenu>)>,
    state: RefCell<State>,
    graph: Graph,
}

declare_class!(
    struct Delegate;

    unsafe impl ClassType for Delegate {
        type Super = NSObject;
        type Mutability = mutability::MainThreadOnly;
        const NAME: &'static str = "Delegate";
    }

    impl DeclaredClass for Delegate {
        type Ivars = DelegateState;
    }

    unsafe impl NSObjectProtocol for Delegate {}

    unsafe impl NSApplicationDelegate for Delegate {
        #[method(applicationDidFinishLaunching:)]
        unsafe fn applicationDidFinishLaunching(&self, _: &NSNotification) {
            let mtm = MainThreadMarker::from(self);
            let app = NSApplication::sharedApplication(mtm);

            for (_, menu) in &self.ivars().main_menus {
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
            #[allow(deprecated)]
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
            let MenuState { submenu, on_hover } = get_menu_state(&menu);
            let id = &submenu.id(&self.ivars().state.borrow());
            let data = &self.ivars().graph[id];

            let mut state = self.ivars().state.borrow_mut();
            state.update(on_hover);

            let items = menu.itemArray();
            let mut iter = items.iter();

            // Hacky code for the main menu having the system menu
            if menu.numberOfItems() as usize != data.entries.len() {
                assert_eq!(menu.numberOfItems() as usize, data.entries.len() + 1);
                let _ = iter.next().unwrap();
            }

            for (item, entry) in iter.zip(&data.entries) {
                item.setHidden(!entry.display.evaluate(&state));
                item.setEnabled(entry.active.evaluate(&state));
            }
        }

        #[method(menuDidClose:)]
        fn _menuDidClose(&self, menu: &NSMenu) {
            let MenuState { submenu, .. } = get_menu_state(&menu);
            let id = &submenu.id(&self.ivars().state.borrow());
            let data = &self.ivars().graph[id];

            self.ivars().state.borrow_mut().update(&data.on_leave);
        }

        #[method(menuNeedsUpdate:)]
        fn _menuNeedsUpdate(&self, menu: &NSMenu) {
            if unsafe { menu.numberOfItems() } > 0 {
                return;
            }
            let mtm = MainThreadMarker::new().unwrap();
            let MenuState { submenu, .. } = get_menu_state(&menu);
            let id = &submenu.id(&self.ivars().state.borrow());
            let data = &self.ivars().graph[id];

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
                                submenu: submenu.clone(),
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
            let MenuState { submenu, .. } = get_menu_state(&menu);
            let id = &submenu.id(&self.ivars().state.borrow());

            match &self.ivars().graph[id].entries[menu.indexOfItem(item) as usize].reaction {
                Reaction::SubMenu { .. } => {
                    unreachable!("found submenu where clickaction was expected")
                }
                Reaction::ClickAction { on_action, act } => {
                    // Update state before doing the action
                    self.ivars().state.borrow_mut().update(&on_action);

                    // Update the main menu.
                    // TODO: Find a better way to do this.
                    self.update_main_menu();

                    if let Some(act) = act {
                        match act {
                            ClickAction::ColapseMenu => {
                                // Do nothing, as the menu will close automatically after an item was clicked
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
        let mtm = MainThreadMarker::from(self);
        let app = NSApplication::sharedApplication(mtm);

        let menu = {
            let state = self.ivars().state.borrow();
            self.ivars()
                .main_menus
                .iter()
                .find_map(|(cond, menu)| cond.evaluate(&state).then_some(menu))
                .expect("could not find a main menu")
        };

        unsafe {
            menu.itemAtIndex(0)
                .unwrap()
                .setSubmenu(Some(&create_system_menu(&app)))
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
    &menu.ivars()
}

impl Delegate {
    fn new(mtm: MainThreadMarker) -> Retained<Self> {
        let data = Data::load();
        let main_menus = data
            .root
            .menu
            .entries
            .into_iter()
            .map(|entry| {
                let state = match &entry.reaction {
                    Reaction::SubMenu { submenu, on_hover } => MenuState {
                        submenu: submenu.clone(),
                        on_hover: on_hover.clone(),
                    },
                    _ => unreachable!(),
                };
                let menu = CustomMenu::new(mtm, state);
                unsafe { menu.setAutoenablesItems(false) };

                (entry.display, menu)
            })
            .collect();
        let this = mtm.alloc().set_ivars(DelegateState {
            main_menus,
            state: RefCell::new(data.root.state),
            graph: data.graph,
        });
        let this: Retained<Self> = unsafe { msg_send_id![super(this), init] };
        for (_, menu) in &this.ivars().main_menus {
            unsafe { menu.setDelegate(Some(ProtocolObject::from_ref(&*this))) };
        }
        this
    }
}

pub struct MenuState {
    submenu: SubMenu,
    on_hover: Action,
}

declare_class!(
    struct CustomMenu;

    unsafe impl ClassType for CustomMenu {
        type Super = NSMenu;
        type Mutability = mutability::MainThreadOnly;
        const NAME: &'static str = "CustomMenu";
    }

    impl DeclaredClass for CustomMenu {
        type Ivars = MenuState;
    }
);

impl CustomMenu {
    fn new(mtm: MainThreadMarker, state: MenuState) -> Retained<Self> {
        let this = mtm.alloc().set_ivars(state);
        unsafe { msg_send_id![super(this), init] }
    }
}

fn create_system_menu(app: &NSApplication) -> Retained<NSMenu> {
    unsafe {
        let mtm = MainThreadMarker::new().unwrap();

        let name = ns_string!(NAME); // NSProcessInfo::processInfo().processName();
        let menu = NSMenu::initWithTitle(mtm.alloc(), &name);

        menu.addItemWithTitle_action_keyEquivalent(
            &NSString::from_str(&format!("About {name}")),
            Some(sel!(orderFrontStandardAboutPanel:)),
            ns_string!(""),
        );
        menu.addItem(&NSMenuItem::separatorItem(mtm));

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
        menu.addItem(&NSMenuItem::separatorItem(mtm));

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
        hide_others.setKeyEquivalentModifierMask(
            NSEventModifierFlags::NSEventModifierFlagCommand
                | NSEventModifierFlags::NSEventModifierFlagOption,
        );
        menu.addItem(&hide_others);
        menu.addItemWithTitle_action_keyEquivalent(
            ns_string!("Show All"),
            Some(sel!(unhideAllApplications:)),
            ns_string!(""),
        );
        menu.addItem(&NSMenuItem::separatorItem(mtm));

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
    let app = NSApplication::sharedApplication(mtm);
    app.setActivationPolicy(NSApplicationActivationPolicy::Regular);
    let delegate = Delegate::new(mtm);
    unsafe { app.setDelegate(Some(ProtocolObject::from_ref(&*delegate))) };

    unsafe { NSApplicationMain(0, NonNull::new((&mut []).as_mut_ptr()).unwrap()) };
}
