import Foundation

struct GameState {
    var saveButton: ClickableMenuItem?
    var insertedDiskInDriveA: Bool
    var darkWebEnabled: Bool

    func open(url: String) {
        // TODO
    }

    mutating func win() {
        saveButton = link("Save", url: "https://xkcd.com/1975/v6xso1_right_click_save.png")
    }
}

func entryPoint(state: GameState) -> [MenuItem] {
    var children: [MenuItem] = [
        menu("File", file),
        button("Edit") {
            // TODO
        },
        menu("System", [
            button("Shut down") {
                // TODO
            },
            menu("/", rootDirectory),
        ]),
        menu("View", [
            button("Cascade") {}, // TODO
            button("Tile") {}, // TODO
            button("Minimize") {}, // TODO
            button("Full Screen") {}, // TODO
        ]),
        menu("Utilities", [
            // TODO
        ]),
        menu("Games", games),
        menu("Help", help(without: nil)),
    ]
    if let saveButton = state.saveButton {
        children.insert(saveButton, at: 0)
    }
    if state.darkWebEnabled {
        children.append(menu("Do Crimes", []))
    }
    return children
}

func file(state: inout GameState) -> [MenuItem] {
    var children: [MenuItem] = [
        button("Close") {
            // Do nothing, as the menu will close automatically after an item was selected
        },
        menu("Open", [
            menu("A:\\") { state in
                if state.insertedDiskInDriveA {
                    [] // TODO
                } else {
                    ["Please insert disk into Drive A", menu("Insert", [
                        button("Floppy disk") { state in
                            state.insertedDiskInDriveA = true
                        },
                        menu("Chip card", [
                            // TODO
                        ]),
                    ])]
                }
            },
            menu("C:\\", cDrive),
            menu("/", rootDirectory),
        ]),
        menu("Find", [
            menu("Where", []),
            menu("When", [
                button("How?!"),
                button("How?!"),
            ]),
            menu("How", [
                button("How?!"),
            ]),
            "What",
            link("Why", url: "https://itisamystery.com/"),
            menu("Who", [
                backAndForth([("'s on First", "todo")], last: link("todo", url: "...")),
                backAndForth([("is the Band on Stage", "todo")], last: link("todo", url: "...")),
            ]),
        ]),
        button("Backup") {
            // TODO
        },
    ]
    if let saveButton = state.saveButton {
        children.append(saveButton)
    }
    return children
}

let music: [MenuItem] = []

let games: [MenuItem] = []

func help(without: String?) -> [MenuItem] {
    var items: [MenuItem] = []
    for title in ["Tutorial", "Support", "Manual", "Troubleshooting", "FAQ", "Guide", "Q&A", "User forums"] {
        if title != without {
            items.append(menu(title) { help(without: title) })
        }
    }
    items.append(separator())
    items.append(menu("Credits", [
        "Some of the people who collaborated on this comic:",
        link("@chromakode", url: "https://chromakode.com/"),
        link("Amber", url: "https://twitter.com/aiiane"),
        link("@fadinginterest", url: "https://twitter.com/fadinginterest"),
        link("Kat", url: "https://twitter.com/wirehead2501"),
        link("Kevin", url: "https://twitter.com/cotrone"),
        link("Stereo", url: "https://90d.ca/"),
    ]))
    return items
}

let rootDirectory: [MenuItem] = [
    menu("home/", [
        button("guest"),
        menu("user", cDrive),
        menu("root", ["You are not in the sudoers file. This incident will be reported."]),
    ]),
    button("opt/"),
    button("sbin/"),
    menu("usr/", usr(without: "usr/")),
    menu("dev/", [
        link("random/", url: "https://c.xkcd.com/random/comic/"),
        link("urandom/", url: "https://c.xkcd.com/random/comic/"),
    ]),
]

func usr(without: String) -> [MenuItem] {
    let paths = ["local/", "bin/", "share/", "opt/", "usr/", "var/", "sbin/"]
    var items: [MenuItem] = []
    for path in paths {
        if path == without {
            continue
        }
        items.append(menu(path) { usr(without: path) })
    }
    return items
}

let cDrive: [MenuItem] = [
    button("Documents\\") {
        // Does nothing?
    },
    menu("Music\\", music),
    menu("Bookmarks\\", [
        menu("Comics", []), // TODO
        menu("Secret") { state in
            if state.darkWebEnabled {
                [button("Enable Dark Web") { state in
                    state.darkWebEnabled = false
                }]
            } else {
                [button("Disable Dark Web") { state in
                    state.darkWebEnabled = true
                }]
            }
        }
    ]),
    menu("Games\\", games),
    backAndForth([
        ("Sequences\\", "Good morning Paul. What will your first sequence of the day be?"),
        ("Celery Man", "CELERY MAN"),
        ("Could you kick up the 4d3d3d3?", "4d3d3d3 engaged."),
        ("add sequence: OYSTER", "OYSTER"),
        ("Uhhh... give me a printout of Oyster smiling.", "*whirrrrrrrr*"),
        ("Computer?", "Yes."),
        ("Do we have any new sequences?", "I have a BETA sequence I've been working on. Would you like to see it?"),
        ("... alright.", "Hey Paul, I'm Tayne, your latest dancer. I can't wait to entertain ya."),
        ("Now Tayne I can get into.", "TAYNE"),
        ("Can I see a hat wobble?", "Yes."),
        ("And a flarhgunnstow?", "Yes."),
        ("Is there any way to generate a nude Tayne?", "Not computing. Please repeat."),
        ("Nude. Tayne.", "This is not suitable for work. Are you sure?"),
    ], last: link("Mmmhmm.", url: "https://www.youtube.com/watch?v=MHWBEK8w_YY")),
]
