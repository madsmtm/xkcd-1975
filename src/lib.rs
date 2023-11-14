use std::collections::HashMap;

use serde_derive::{Deserialize, Serialize};

/// An identifier for menus in the graph.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Deserialize, Serialize)]
#[serde(transparent)]
pub struct MenuId(String);

pub type Graph = HashMap<MenuId, Menu>;

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct Data {
    pub root: Root,
    pub graph: Graph,
}

impl Data {
    /// The source data, extracted from <https://xkcd.com/s/f9dfe4.js>.
    const JSON: &str = include_str!("data.json");

    /// Load the data from source.
    pub fn load() -> Self {
        serde_json::from_str(Self::JSON).unwrap()
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct Root {
    #[serde(rename = "State")]
    pub state: State,
    #[serde(rename = "Menu")]
    pub menu: Menu,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct State {
    #[serde(rename = "Tags")]
    tags: HashMap<MenuId, String>,
}

impl State {
    pub fn update(&mut self, action: &Action) {
        // Explicitly set tags before unsetting; this is the same order as in the original source
        self.tags.extend(action.set_tags.clone());
        self.tags.retain(|k, _| !action.unset_tags.contains(k));
        if !action.set_tags.is_empty() || !action.unset_tags.is_empty() {
            eprintln!(
                "Updated tags: {:#?}",
                self.tags.keys().map(|tag| &tag.0).collect::<Vec<_>>()
            );
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize, Default)]
pub struct Action {
    #[serde(rename = "setTags")]
    set_tags: HashMap<MenuId, String>,
    #[serde(rename = "unsetTags")]
    unset_tags: Vec<MenuId>,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct MenuItem {
    /// Unused.
    icon: Option<String>,
    /// The string shown on the menu item.
    pub label: String,
    /// Whether the menu item should be shown.
    pub display: Conditional,
    /// Whether the menu item should be clickable / not disabled.
    pub active: Conditional,
    /// What should be done when the user hovers and/or clicks on the menu item.
    pub reaction: Reaction,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[serde(tag = "tag")]
pub enum Conditional {
    Always,
    TagSet { contents: MenuId },
    TagUnset { contents: MenuId },
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
pub enum Reaction {
    SubMenu {
        #[serde(rename = "onAction")]
        on_hover: Action,
        #[serde(flatten)]
        submenu: SubMenu,
    },
    #[serde(rename = "Action")]
    ClickAction {
        #[serde(rename = "onAction")]
        on_action: Action,
        act: Option<ClickAction>,
    },
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct SubMenu {
    #[serde(rename = "subMenu")]
    sub_menu: MenuId,
    #[serde(rename = "subIdPostfix")]
    sub_id_postfix: Option<MenuId>,
}

impl SubMenu {
    pub fn id(&self, state: &State) -> MenuId {
        if let Some(postfix_id) = &self.sub_id_postfix {
            if let Some(postfix) = state.tags.get(postfix_id) {
                MenuId(format!("{}{}", self.sub_menu.0, postfix))
            } else {
                // Fall back to no postfix if no tag with that ID was set.
                self.sub_menu.clone()
            }
        } else {
            self.sub_menu.clone()
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[serde(tag = "tag")]
pub enum ClickAction {
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
pub struct Menu {
    pub id: MenuId,
    #[serde(rename = "onLeave")]
    pub on_leave: Action,
    pub entries: Vec<MenuItem>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn everything_is_parsed() {
        let expected: serde_json::Value = serde_json::from_str(Data::JSON).unwrap();

        let parsed = Data::load();
        let after_roundtrip = serde_json::to_value(parsed).unwrap();

        assert_eq!(expected, after_roundtrip);
    }

    #[test]
    fn root_menu_is_same_as_in_graph() {
        let data = Data::load();

        assert_eq!(data.root.menu, data.graph[&data.root.menu.id]);
    }
}
