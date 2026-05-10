use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use toml::Value;

use super::{
    UiAssetHeader, UiAssetImports, UiComponentParamSchema, UiComponentPublicContract,
    UiNamedSlotSchema, UiNodeDefinitionKind, UiStyleDeclarationBlock, UiStyleScope, UiStyleSheet,
};
use crate::ui::accessibility::UiAccessibilityContract;
use crate::ui::focus::UiFocusContract;
use crate::ui::navigation::UiNavigationContract;
use crate::ui::picking::UiPickPolicy;
use crate::ui::template::UiBindingRef;
use crate::ui::widget::UiWidgetContract;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct UiPrototypeNodeHandle(pub u32);

impl UiPrototypeNodeHandle {
    pub const fn new(index: u32) -> Self {
        Self(index)
    }

    pub const fn index(self) -> usize {
        self.0 as usize
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiRawAssetPrototype {
    pub asset: UiAssetHeader,
    #[serde(default)]
    pub imports: UiAssetImports,
    #[serde(default)]
    pub tokens: BTreeMap<String, Value>,
    pub document: UiDocumentPrototype,
    #[serde(default)]
    pub components: BTreeMap<String, UiComponentPrototype>,
    #[serde(default)]
    pub styles: Vec<UiStylePrototype>,
}

impl UiRawAssetPrototype {
    pub fn node(&self, handle: UiPrototypeNodeHandle) -> Option<&UiNodePrototype> {
        self.document.nodes.get(handle.index())
    }

    pub fn node_count(&self) -> usize {
        self.document.nodes.len()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiDocumentPrototype {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root: Option<UiPrototypeNodeHandle>,
    #[serde(default)]
    pub nodes: Vec<UiNodePrototype>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiComponentPrototype {
    pub root: UiPrototypeNodeHandle,
    #[serde(default)]
    pub style_scope: UiStyleScope,
    #[serde(default)]
    pub contract: UiComponentPublicContract,
    #[serde(default)]
    pub params: BTreeMap<String, UiComponentParamSchema>,
    #[serde(default)]
    pub slots: BTreeMap<String, UiNamedSlotSchema>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiStylePrototype {
    pub stylesheet: UiStyleSheet,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiNodePrototype {
    #[serde(default)]
    pub node_id: String,
    #[serde(default)]
    pub kind: UiNodeDefinitionKind,
    #[serde(default, rename = "type")]
    pub widget_type: Option<String>,
    #[serde(default)]
    pub component: Option<String>,
    #[serde(default)]
    pub component_ref: Option<String>,
    #[serde(default)]
    pub slot_name: Option<String>,
    #[serde(default)]
    pub control_id: Option<String>,
    #[serde(default)]
    pub classes: Vec<String>,
    #[serde(default)]
    pub params: BTreeMap<String, Value>,
    #[serde(default)]
    pub props: BTreeMap<String, Value>,
    #[serde(default)]
    pub layout: Option<BTreeMap<String, Value>>,
    #[serde(default)]
    pub bindings: Vec<UiBindingRef>,
    #[serde(default)]
    pub style_overrides: UiStyleDeclarationBlock,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub focus: Option<UiFocusContract>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub navigation: Option<UiNavigationContract>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub picking: Option<UiPickPolicy>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub a11y: Option<UiAccessibilityContract>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub widget: Option<UiWidgetContract>,
    #[serde(default)]
    pub children: Vec<UiPrototypeChildMount>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiPrototypeChildMount {
    #[serde(default)]
    pub mount: Option<String>,
    #[serde(default)]
    pub slot: BTreeMap<String, Value>,
    pub child: UiPrototypeNodeHandle,
}
