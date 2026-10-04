use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::ui::template::{UiAssetFingerprint, UiInvalidationSnapshot};

/// 编译结果的缓存身份，包含源文档、已解析导入及影响编译结果的外部修订。
/// 运行时以完整键查找缓存；`invalidation_snapshot` 把同一输入向量交给失效图解释未命中。
/// 扩展编译输入时须同时更新键的构造和快照投影。
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct UiCompileCacheKey {
    pub root_document: UiAssetFingerprint,
    pub widget_imports: BTreeMap<String, UiAssetFingerprint>,
    pub style_imports: BTreeMap<String, UiAssetFingerprint>,
    #[serde(default)]
    pub declared_widget_imports_revision: UiAssetFingerprint,
    #[serde(default)]
    pub declared_style_imports_revision: UiAssetFingerprint,
    pub descriptor_registry_revision: u64,
    pub component_contract_revision: UiAssetFingerprint,
    pub resource_dependencies_revision: UiAssetFingerprint,
}

impl UiCompileCacheKey {
    /// 把缓存身份中的全部修订投影给失效图；新增编译输入时，键与快照必须同步扩展。
    pub fn invalidation_snapshot(&self) -> UiInvalidationSnapshot {
        UiInvalidationSnapshot {
            document: self.root_document,
            widget_imports: self.widget_imports.clone(),
            style_imports: self.style_imports.clone(),
            declared_widget_imports_revision: self.declared_widget_imports_revision,
            declared_style_imports_revision: self.declared_style_imports_revision,
            descriptor_registry_revision: self.descriptor_registry_revision,
            component_contract_revision: self.component_contract_revision,
            resource_dependencies_revision: self.resource_dependencies_revision,
        }
    }
}
