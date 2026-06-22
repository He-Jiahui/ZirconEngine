use zircon_runtime_interface::ui::component::UiDefaultNodeTemplate;

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum UiAssetPaletteEntryKind {
    Native {
        widget_type: String,
        default_node: UiDefaultNodeTemplate,
    },
    Component {
        component: String,
    },
    Reference {
        component_ref: String,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct UiAssetPaletteEntry {
    pub label: String,
    pub kind: UiAssetPaletteEntryKind,
}
