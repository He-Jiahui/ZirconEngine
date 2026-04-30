use slint::{Image, SharedString};

#[derive(Clone, Default)]
pub(crate) struct AssetFolderData {
    pub id: SharedString,
    pub name: SharedString,
    pub count: i32,
    pub depth: i32,
    pub selected: bool,
}

#[derive(Clone, Default)]
pub(crate) struct AssetItemData {
    pub uuid: SharedString,
    pub locator: SharedString,
    pub name: SharedString,
    pub file_name: SharedString,
    pub kind: SharedString,
    pub extension: SharedString,
    pub dirty: bool,
    pub has_error: bool,
    pub has_preview: bool,
    pub state: SharedString,
    pub revision: SharedString,
    pub selected: bool,
    pub preview: Image,
}

#[derive(Clone, Default)]
pub(crate) struct AssetReferenceData {
    pub uuid: SharedString,
    pub locator: SharedString,
    pub name: SharedString,
    pub kind: SharedString,
    pub known_project_asset: bool,
}

#[derive(Clone, Default)]
pub(crate) struct AssetSelectionData {
    pub uuid: SharedString,
    pub name: SharedString,
    pub locator: SharedString,
    pub kind: SharedString,
    pub meta_path: SharedString,
    pub adapter_key: SharedString,
    pub state: SharedString,
    pub revision: SharedString,
    pub diagnostics: SharedString,
    pub has_preview: bool,
    pub preview: Image,
}
