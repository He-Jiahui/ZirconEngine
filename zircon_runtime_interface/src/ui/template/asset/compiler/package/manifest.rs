use serde::{Deserialize, Serialize};

use crate::ui::template::{
    UiAssetFingerprint, UiAssetKind, UiLocalizationDependency, UiResourceDependency,
};

/// 编译时解析所得的依赖快照，随包报告记录导入资产、资源和本地化引用。
/// 运行时编译器负责填充它；条目不保证当前磁盘状态仍与记录的指纹一致。
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
