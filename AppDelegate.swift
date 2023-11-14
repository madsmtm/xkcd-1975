import AppKit

class AppDelegate: NSObject, NSApplicationDelegate {
    func applicationDidFinishLaunching(_ notification: Notification) {
        // The built-in menu, not part of the original game
        let name = ProcessInfo.processInfo.processName
        let system = NSMenu(title: name)
        let systemItem = NSMenuItem(title: name, action: nil, keyEquivalent: "")
        systemItem.submenu = system

        system.addItem(withTitle: "About \(name)", action: #selector(NSApplication.orderFrontStandardAboutPanel(_:)), keyEquivalent: "")
        system.addItem(NSMenuItem.separator())

        let services = NSMenu(title: "Services")
        let servicesItem = NSMenuItem(title: "Services", action: nil, keyEquivalent: "")
        servicesItem.submenu = services
        NSApp.servicesMenu = services
        system.addItem(servicesItem)
        system.addItem(NSMenuItem.separator())

        system.addItem(withTitle: "Hide \(name)", action: #selector(NSApplication.hide(_:)), keyEquivalent: "h")
        let hideOthers = NSMenuItem(title: "Hide Others", action: #selector(NSApplication.hideOtherApplications(_:)), keyEquivalent: "h")
        hideOthers.keyEquivalentModifierMask = [.command, .option]
        system.addItem(hideOthers)
        system.addItem(withTitle: "Show All", action: #selector(NSApplication.unhideAllApplications(_:)), keyEquivalent: "")
        system.addItem(NSMenuItem.separator())

        system.addItem(withTitle: "Ouit \(name)", action: #selector(NSApplication.terminate(_:)), keyEquivalent: "q")

        // The current main menu
        let main = NSMenu()
        main.addItem(systemItem)
        NSApp.mainMenu = main
    }

    // Required to prevent a warning regarding saved application state
    // https://sector7.computest.nl/post/2022-08-process-injection-breaking-all-macos-security-layers-with-a-single-vulnerability/
    func applicationSupportsSecureRestorableState(_ app: NSApplication) -> Bool {
        return true
    }
}
