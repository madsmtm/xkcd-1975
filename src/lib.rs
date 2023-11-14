use std::collections::HashMap;

use serde_derive::{Deserialize, Serialize};

/// The source data, extracted from <https://xkcd.com/s/f9dfe4.js>.
pub const DATA_JSON: &str = include_str!("data.json");

/// An identifier for menus in the graph.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Deserialize, Serialize)]
#[serde(transparent)]
struct Id(String);

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct Data {
    pub root: Root,
    graph: HashMap<Id, Menu>,
}

impl Data {
    pub fn load() -> serde_json::Result<Self> {
        serde_json::from_str(DATA_JSON)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct Root {
    #[serde(rename = "State")]
    pub state: State,
    #[serde(rename = "Menu")]
    menu: Menu,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct State {
    #[serde(rename = "Tags")]
    tags: HashMap<Id, String>,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct Action {
    #[serde(rename = "setTags")]
    set_tags: HashMap<Id, String>,
    #[serde(rename = "unsetTags")]
    unset_tags: Vec<Id>,
}

impl Action {
    pub fn update_state(&self, state: &mut State) {
        // Explicitly set tags before unsetting; this is the same order as in the original source
        state.tags.extend(self.set_tags.clone());
        state.tags.retain(|k, _| self.unset_tags.contains(k));
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
struct MenuItem {
    /// Unused.
    icon: Option<String>,
    /// The string shown on the menu item.
    label: String,
    /// Whether the menu item should be shown.
    display: Conditional,
    /// Whether the menu item should be clickable / not disabled.
    active: Conditional,
    /// What should be done when the user hovers and/or clicks on the menu item.
    reaction: Reaction,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[serde(tag = "tag")]
enum Conditional {
    Always,
    TagSet { contents: Id },
    TagUnset { contents: Id },
    TLNot { contents: Box<Conditional> },
    TLAnd { contents: Vec<Conditional> },
    TLOr { contents: Vec<Conditional> },
}

impl Conditional {
    pub fn evaluate(&self, state: &State) -> bool {
        match self {
            Self::Always => true,
            Self::TagSet { contents } => state.tags.contains_key(contents),
            Self::TagUnset { contents } => !state.tags.contains_key(contents),
            Self::TLNot { contents } => !contents.evaluate(state),
            Self::TLAnd { contents } => contents.iter().all(|item| item.evaluate(state)),
            Self::TLOr { contents } => contents.iter().any(|item| item.evaluate(state)),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[serde(tag = "tag")]
enum Reaction {
    SubMenu {
        #[serde(rename = "onAction")]
        on_action: Action,
        #[serde(rename = "subMenu")]
        sub_menu: Id,
        #[serde(rename = "subIdPostfix")]
        sub_id_postfix: Option<Id>,
    },
    #[serde(rename = "Action")]
    ClickAction {
        #[serde(rename = "onAction")]
        on_action: Action,
        act: Option<ClickAction>,
    },
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[serde(tag = "tag")]
enum ClickAction {
    ColapseMenu,
    Nav {
        url: String,
    },
    Download {
        url: String,
        filename: String,
    },
    JSCall {
        #[serde(rename = "jsCall")]
        js_call: String,
    },
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
struct Menu {
    id: Id,
    #[serde(rename = "onLeave")]
    on_leave: Action,
    entries: Vec<MenuItem>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn everything_is_parsed() {
        let expected: serde_json::Value = serde_json::from_str(DATA_JSON).unwrap();

        let parsed = Data::load().unwrap();
        let after_roundtrip = serde_json::to_value(parsed).unwrap();

        assert_eq!(expected, after_roundtrip);
    }

    #[test]
    fn root_menu_is_same_as_in_graph() {
        let data = Data::load().unwrap();

        assert_eq!(data.root.menu, data.graph[&data.root.menu.id]);
    }
}
