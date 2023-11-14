import Foundation

protocol MenuItem {
    /// The text displayed for this menu item.
    var title: String { get }
}

extension String: MenuItem {
    var title: String {
        return self
    }
}

protocol ClickableMenuItem: MenuItem {
    /// Called when the user clicks on this item.
    func click(state: inout GameState)
}

protocol MenuItemWithChildren: MenuItem {
    /// Retrieved on hover.
    func children(state: inout GameState) -> [MenuItem]
}

struct button: ClickableMenuItem {
    let title: String
    let onClick: (inout GameState) -> Void

    func click(state: inout GameState) {
        onClick(&state)
    }

    init(_ title: String) {
        self.title = title
        self.onClick = { state in }
    }

    init(_ title: String, onClick: @escaping (inout GameState) -> Void) {
        self.title = title
        self.onClick = onClick
    }

    init(_ title: String, onClick: @escaping () -> Void) {
        self.init(title) { (gameState) -> Void in
            onClick()
        }
    }
}

struct menu: MenuItemWithChildren {
    let title: String
    let getChildren: (inout GameState) -> [MenuItem]

    func children(state: inout GameState) -> [MenuItem] {
        return getChildren(&state)
    }

    init(_ title: String, _ children: [MenuItem]) {
        self.title = title
        self.getChildren = { state in children }
    }

    init(_ title: String, _ getChildren: @escaping () -> [MenuItem]) {
        self.title = title
        self.getChildren = { state in getChildren() }
    }

    init(_ title: String, _ getChildren: @escaping (inout GameState) -> [MenuItem]) {
        self.title = title
        self.getChildren = getChildren
    }
}

func backAndForth(_ qAndA: [(String, String)], last: MenuItem) -> MenuItem {
    var current: MenuItem = last
    for (q, a) in qAndA.reversed() {
        current = menu(q, [a, current])
    }
    return current
}

func link(_ title: String, url: String) -> button {
    button(title) { state in
        state.open(url: url)
    }
}

func separator() -> MenuItem {
    return "--------------"
}
