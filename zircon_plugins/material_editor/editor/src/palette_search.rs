use zircon_editor::core::editor_authoring_extension::{
    GraphNodeDescriptor, GraphNodePaletteDescriptor,
};

/// Searches only the actions registered by the real material backend.
///
/// Tokens are ANDed across the stable action key, caption and category. The
/// declaration order is retained; this does not implement UE's relevance scorer.
pub fn search_material_graph_node_palette(query: &str) -> Vec<GraphNodeDescriptor> {
    matching_actions(&super::plugin::material_node_palette(), query)
}

pub(super) fn matching_actions(
    palette: &GraphNodePaletteDescriptor,
    query: &str,
) -> Vec<GraphNodeDescriptor> {
    let tokens: Vec<_> = query.split_whitespace().map(str::to_lowercase).collect();
    palette
        .nodes()
        .iter()
        .filter(|node| {
            let fields = [
                node.id().to_lowercase(),
                node.display_name().to_lowercase(),
                node.category().to_lowercase(),
            ];
            tokens
                .iter()
                .all(|token| fields.iter().any(|field| field.contains(token.as_str())))
        })
        .cloned()
        .collect()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MaterialGraphNodeSearchNavigation {
    Previous,
    Next,
    PageUp,
    PageDown,
    First,
    Last,
}

/// Ephemeral action-menu state; it neither owns a graph document nor edits one.
///
/// Activation yields a registered action descriptor once. The host must resolve
/// its real creation payload, document/revision and graph-local anchor, then use
/// the existing prepared domain delta/history. This is not a mutation permit.
/// Focus restoration belongs to the host that opened the popup.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaterialGraphNodeSearch {
    query: String,
    matches: Vec<GraphNodeDescriptor>,
    selected_id: Option<String>,
    open: bool,
}

impl Default for MaterialGraphNodeSearch {
    fn default() -> Self {
        Self::new("")
    }
}

impl MaterialGraphNodeSearch {
    pub fn new(query: impl Into<String>) -> Self {
        let query = query.into();
        let matches = search_material_graph_node_palette(&query);
        let selected_id = matches.first().map(|node| node.id().to_owned());
        Self {
            query,
            matches,
            selected_id,
            open: true,
        }
    }

    pub fn query(&self) -> &str {
        &self.query
    }

    pub fn matches(&self) -> &[GraphNodeDescriptor] {
        &self.matches
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn selected(&self) -> Option<&GraphNodeDescriptor> {
        if !self.open {
            return None;
        }
        let id = self.selected_id.as_deref()?;
        self.matches.iter().find(|node| node.id() == id)
    }

    pub fn set_query(&mut self, query: impl Into<String>) {
        if !self.open {
            return;
        }
        self.query = query.into();
        self.matches = search_material_graph_node_palette(&self.query);
        if self.selected().is_none() {
            self.selected_id = self.matches.first().map(|node| node.id().to_owned());
        }
    }

    /// A stale or filtered-out row must not activate a different action.
    pub fn select_id(&mut self, id: &str) -> bool {
        if !self.open || !self.matches.iter().any(|node| node.id() == id) {
            return false;
        }
        self.selected_id = Some(id.to_owned());
        true
    }

    pub fn navigate(&mut self, navigation: MaterialGraphNodeSearchNavigation) {
        if !self.open || self.matches.is_empty() {
            return;
        }
        let current = self
            .matches
            .iter()
            .position(|node| Some(node.id()) == self.selected_id.as_deref())
            .unwrap_or(0);
        let last = self.matches.len() - 1;
        // UE SGraphActionMenu uses a 15-action page step. Clamp at both ends.
        let next = match navigation {
            MaterialGraphNodeSearchNavigation::Previous => current.saturating_sub(1),
            MaterialGraphNodeSearchNavigation::Next => current.saturating_add(1).min(last),
            MaterialGraphNodeSearchNavigation::PageUp => current.saturating_sub(15),
            MaterialGraphNodeSearchNavigation::PageDown => current.saturating_add(15).min(last),
            MaterialGraphNodeSearchNavigation::First => 0,
            MaterialGraphNodeSearchNavigation::Last => last,
        };
        self.selected_id = Some(self.matches[next].id().to_owned());
    }

    /// Activates the clicked action only while its stable ID is still visible.
    ///
    /// A filtered-out or closed row leaves the current selection and popup alone.
    /// The host must separately guard the originating popup/document incarnation;
    /// this menu-local check does not authorize an edit against another revision.
    pub fn activate_id(&mut self, id: &str) -> Option<GraphNodeDescriptor> {
        if !self.select_id(id) {
            return None;
        }
        self.activate()
    }

    pub fn activate(&mut self) -> Option<GraphNodeDescriptor> {
        let action = self.selected()?.clone();
        self.cancel();
        Some(action)
    }

    pub fn cancel(&mut self) {
        self.open = false;
        self.selected_id = None;
    }
}
