import AppKit

class AppDelegate: NSObject, NSApplicationDelegate {
    /// The built-in menu, not part of the original game.
    var system: NSMenuItem!

    func applicationDidFinishLaunching(_ notification: Notification) {
        let name = ProcessInfo.processInfo.processName
        let systemMenu = NSMenu(title: name)
        system = NSMenuItem(title: name, action: nil, keyEquivalent: "")
        system.submenu = systemMenu

        systemMenu.addItem(withTitle: "About \(name)", action: #selector(NSApplication.orderFrontStandardAboutPanel(_:)), keyEquivalent: "")
        systemMenu.addItem(NSMenuItem.separator())

        let services = NSMenu(title: "Services")
        let servicesItem = NSMenuItem(title: "Services", action: nil, keyEquivalent: "")
        servicesItem.submenu = services
        NSApp.servicesMenu = services
        systemMenu.addItem(servicesItem)
        systemMenu.addItem(NSMenuItem.separator())

        systemMenu.addItem(withTitle: "Hide \(name)", action: #selector(NSApplication.hide(_:)), keyEquivalent: "h")
        let hideOthers = NSMenuItem(title: "Hide Others", action: #selector(NSApplication.hideOtherApplications(_:)), keyEquivalent: "h")
        hideOthers.keyEquivalentModifierMask = [.command, .option]
        systemMenu.addItem(hideOthers)
        systemMenu.addItem(withTitle: "Show All", action: #selector(NSApplication.unhideAllApplications(_:)), keyEquivalent: "")
        systemMenu.addItem(NSMenuItem.separator())

        systemMenu.addItem(withTitle: "Ouit \(name)", action: #selector(NSApplication.terminate(_:)), keyEquivalent: "q")

        // The current main menu
        let main = NSMenu()
        main.addItem(system)
        NSApp.mainMenu = main
    }

    // Required to prevent a warning regarding saved application state
    // https://sector7.computest.nl/post/2022-08-process-injection-breaking-all-macos-security-layers-with-a-single-vulnerability/
    func applicationSupportsSecureRestorableState(_ app: NSApplication) -> Bool {
        return true
    }
}

let app = NSApplication.shared
let delegate = AppDelegate()
app.delegate = delegate

_ = NSApplicationMain(CommandLine.argc, CommandLine.unsafeArgv)
