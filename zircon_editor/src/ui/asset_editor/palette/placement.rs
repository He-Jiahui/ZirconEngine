use std::collections::BTreeMap;

use toml::Value;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PaletteInsertMode {
    Child,
    After,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct UiAssetPaletteInsertionPlacement {
    pub mount: Option<String>,
    pub slot: BTreeMap<String, Value>,
}
