use std::cell::RefCell;
use std::ptr::NonNull;

use icrate::AppKit::{
    NSApplication, NSApplicationActivationPolicyRegular, NSApplicationDelegate, NSApplicationMain,
};
use icrate::Foundation::{NSObject, NSObjectProtocol};
use objc2::declare::{Ivar, IvarDrop};
use objc2::rc::Id;
use objc2::runtime::ProtocolObject;
use objc2::{declare_class, extern_methods, msg_send, mutability, ClassType};

use xkcd_1975::{Data, Graph, State};

pub struct DelegateState {
    state: RefCell<State>,
    // main_menus: Id<NSArray<NSMenu>>,
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
            let this: Option<&mut Self> = unsafe { msg_send![super(this), init] };
            this.map(|this| {
                let data = Data::load();
                Ivar::write(
                    &mut this.state,
                    Box::new(DelegateState {
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
        // Required to prevent a warning regarding saved application state
        // https://sector7.computest.nl/post/2022-08-process-injection-breaking-all-macos-security-layers-with-a-single-vulnerability/
        #[method(applicationSupportsSecureRestorableState:)]
        fn application_supports_secure_restorable_state(&self, _: &NSApplication) -> bool {
            true
        }
    }
);

extern_methods!(
    unsafe impl Delegate {
        #[method_id(new)]
        fn new() -> Id<Self>;
    }
);

fn main() {
    let app = unsafe { NSApplication::sharedApplication() };
    unsafe { app.setActivationPolicy(NSApplicationActivationPolicyRegular) };
    let delegate = Delegate::new();
    unsafe { app.setDelegate(Some(ProtocolObject::from_ref(&*delegate))) };

    unsafe { NSApplicationMain(0, NonNull::new((&mut []).as_mut_ptr()).unwrap()) };
}
