use serde::{Deserialize, Serialize};

use super::resource_ref::UiResourceRef;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum UiResourceDependencySource {
    DocumentImport,
    NodeProp,
    NodeLayout,
    NodeStyleOverride,
    ChildMountSlot,
    StyleRuleDeclaration,
    TokenValue,
    ImportedWidget,
    ImportedStyle,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct UiResourceDependency {
    pub reference: UiResourceRef,
    pub source: UiResourceDependencySource,
    pub path: String,
}
