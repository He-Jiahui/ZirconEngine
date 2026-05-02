use serde::{Deserialize, Serialize};

use crate::ui::template::{
    UiAssetFingerprint, UiAssetKind, UiLocalizationDependency, UiResourceDependency,
};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiCompiledAssetDependencyManifest {
    pub widget_imports: Vec<UiCompiledAssetDependency>,
    pub style_imports: Vec<UiCompiledAssetDependency>,
    pub resource_dependencies: Vec<UiResourceDependency>,
    pub localization_dependencies: Vec<UiLocalizationDependency>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiCompiledAssetDependency {
    pub reference: String,
    pub asset_id: String,
    pub asset_kind: UiAssetKind,
    pub source_schema_version: u32,
    pub fingerprint: UiAssetFingerprint,
}
