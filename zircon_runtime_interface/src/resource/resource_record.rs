use serde::{Deserialize, Serialize};

use super::{
    ResourceDiagnostic, ResourceDiagnosticSeverity, ResourceId, ResourceKind, ResourceLocator,
    ResourceState,
};

/// 资源注册表的可序列化状态快照，汇集身份、修订、依赖、导入指纹和诊断。
/// 构造器只建立 Pending 初值；状态迁移和修订由资源管理器的提交路径约束。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 传递资源待提交元数据或已发布的记录快照，汇集身份、地址、内容身份、依赖和诊断。
///
/// # 字段与状态
///
/// `id`、`kind` 与主定位符标识候选；产物定位符可缺失。`revision` 和 `state` 由提交路径约束，内容身份字段与依赖由导入端提供，诊断供展示端读取。
///
/// # 构造与职责
///
/// 构造器建立 Pending、修订零的候选；各构建器只替换字段。字段公开且可序列化，构造或反序列化成功不证明状态迁移、内容身份或引用关系已通过管理器预检。
///
/// # 生命周期
///
/// 快照拥有字符串、定位符及列表，之后的提交可产生新记录。需要权威当前状态时重新查询；持有旧值不阻止后续发布。
///
/// # 参考
///
/// [`Self::new`]、[`Self::with_state`]、[`Self::failure_reason`]、[`crate::resource::ResourceEvent`]。
pub struct ResourceRecord {
    pub id: ResourceId,
    pub kind: ResourceKind,
    pub primary_locator: ResourceLocator,
    pub artifact_locator: Option<ResourceLocator>,
    pub revision: u64,
    pub state: ResourceState,
    pub dependency_ids: Vec<ResourceId>,
    pub diagnostics: Vec<ResourceDiagnostic>,
    pub source_hash: String,
    pub importer_id: String,
    pub importer_version: u32,
    pub config_hash: String,
}

impl ResourceRecord {
    /// 为导入或登记流程建立尚未提交的 Pending 记录候选。
    ///
    /// # 参数
    ///
    /// `id`、`kind`、`primary_locator` 分别提供身份、种类和当前逻辑地址；调用端负责正确配对，构造器不查询注册表。
    ///
    /// # 返回与后续操作
    ///
    /// 返回修订零、无产物、空依赖和诊断、尚未填写内容身份的记录。导入端按实际结果补字段，再交管理器提交；此处不加载或发布资源。
    ///
    /// # 参考
    ///
    /// [`Self::with_artifact_locator`]、[`Self::with_state`]、[`crate::resource::ResourceId::from_asset_uuid`]。
    pub fn new(id: ResourceId, kind: ResourceKind, primary_locator: ResourceLocator) -> Self {
        Self {
            id,
            kind,
            primary_locator,
            artifact_locator: None,
            revision: 0,
            state: ResourceState::Pending,
            dependency_ids: Vec::new(),
            diagnostics: Vec::new(),
            source_hash: String::new(),
            importer_id: String::new(),
            importer_version: 0,
            config_hash: String::new(),
        }
    }

    pub fn id(&self) -> ResourceId {
        self.id
    }

    pub fn primary_locator(&self) -> &ResourceLocator {
        &self.primary_locator
    }

    pub fn artifact_locator(&self) -> Option<&ResourceLocator> {
        self.artifact_locator.as_ref()
    }

    pub fn with_artifact_locator(mut self, artifact_locator: ResourceLocator) -> Self {
        self.artifact_locator = Some(artifact_locator);
        self
    }

    pub fn with_source_hash(mut self, source_hash: impl Into<String>) -> Self {
        self.source_hash = source_hash.into();
        self
    }

    pub fn with_importer_version(mut self, importer_version: u32) -> Self {
        self.importer_version = importer_version;
        self
    }

    pub fn with_importer_id(mut self, importer_id: impl Into<String>) -> Self {
        self.importer_id = importer_id.into();
        self
    }

    pub fn with_config_hash(mut self, config_hash: impl Into<String>) -> Self {
        self.config_hash = config_hash.into();
        self
    }

    /// 在待提交记录中表达期望阶段，供管理器按相应提交入口验证。
    ///
    /// # 参数
    ///
    /// `state` 只替换候选字段，不自动选择加载、重载或导入恢复入口。
    ///
    /// # 返回与验证
    ///
    /// 返回修改后的记录；这里不拒绝非法迁移、不改变修订或载荷，也不发布事件。提交端负责按操作类型约束迁移与恢复授权。
    ///
    /// # 参考
    ///
    /// [`crate::resource::ResourceState`]、[`Self::new`]、[`Self::with_diagnostics`]。
    pub fn with_state(mut self, state: ResourceState) -> Self {
        self.state = state;
        self
    }

    /// 把本次导入或重载的完整诊断集装入候选记录，替代旧诊断。
    ///
    /// # 参数
    ///
    /// `diagnostics` 被移入记录。消息严重级别与记录状态分别提供，不因存在错误级别诊断而自动修改阶段。
    ///
    /// # 返回与后续操作
    ///
    /// 返回替换诊断集的候选，不在这里提交失败或改变载荷；展示摘要仍受记录状态筛选。
    ///
    /// # 参考
    ///
    /// [`Self::failure_reason`]、[`crate::resource::ResourceDiagnostic`]、[`Self::with_state`]。
    pub fn with_diagnostics(mut self, diagnostics: Vec<ResourceDiagnostic>) -> Self {
        self.diagnostics = diagnostics;
        self
    }

    pub fn with_dependency_ids(mut self, dependency_ids: Vec<ResourceId>) -> Self {
        self.dependency_ids = dependency_ids;
        self
    }

    /// 仅在 Error 状态提供面向用户的摘要：优先首个错误诊断，再退回首条诊断。
    /// 为错误阶段的记录提供可展示的诊断摘要，不把其它阶段的诊断误当成状态失败。
    ///
    /// # 返回与生命周期
    ///
    /// 非 Error 阶段或无诊断时返回 None。Error 阶段优先借用首个错误级别消息，否则借用首条诊断；返回文本随记录存活，不反映后续提交的新记录。
    ///
    /// # 参考
    ///
    /// [`crate::resource::ResourceState`]、[`crate::resource::ResourceDiagnosticSeverity`]、[`Self::with_diagnostics`]。
    pub fn failure_reason(&self) -> Option<&str> {
        if self.state != ResourceState::Error {
            return None;
        }

        self.diagnostics
            .iter()
            .find(|diagnostic| diagnostic.severity == ResourceDiagnosticSeverity::Error)
            .or_else(|| self.diagnostics.first())
            .map(|diagnostic| diagnostic.message.as_str())
    }
}
