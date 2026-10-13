use serde::{Deserialize, Serialize};

use super::{
    ReflectError, ReflectScriptVisibility, ReflectTypeInfo, ReflectTypePath, ReflectTypeRole,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 类型注册携带的声明式序列化策略；具体持久化操作由消费该注册的系统负责。
pub enum ReflectSerializationStrategy {
    /// 未声明序列化策略。
    None,
    /// 使用反射值层的通用值策略。
    Value,
    /// 使用 JSON 表示策略。
    Json,
    /// 使用资源句柄策略。
    ResourceHandle,
    /// 使用实体引用策略。
    EntityReference,
}

/// 中立的类型描述；运行时注册端再校验字段语义并绑定组件或资源适配器。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReflectTypeRegistration {
    /// 经构造/反序列化校验的完整类型路径及可选归属键。
    pub type_path: ReflectTypePath,
    /// 类型面向工具的显示名称。
    pub display_name: String,
    /// 可选类型说明。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub documentation: Option<String>,
    /// 类型形状与当前注册顺序中的字段元数据。
    pub type_info: ReflectTypeInfo,
    /// 声明式序列化策略。
    pub serialization: ReflectSerializationStrategy,
    /// Runtime 绑定的类型角色；具体适配器仍由运行时注册记录提供。
    pub role: ReflectTypeRole,
    /// 是否允许类型级序列化投影；缺省由 `serialization` 的初始值推导。
    pub serializable: bool,
    /// 是否包含在 Editor 可见 schema 查询中。
    pub editor_visible: bool,
    /// 是否包含在远程可见 schema 查询中。
    pub remote_visible: bool,
    /// 是否包含在脚本宿主的类型投影中；与 Editor/远程可见标记独立。
    #[serde(default)]
    pub script_visibility: ReflectScriptVisibility,
}

impl ReflectTypeRegistration {
    /// 创建中立类型注册数据；角色默认为 Value，Editor 可见，远程不可见，脚本私有。
    ///
    /// `serializable` 初始值根据显式 `serialization` 是否为 `None` 计算；Runtime 注册端负责目录接纳校验。
    pub fn new(
        type_path: ReflectTypePath,
        display_name: impl Into<String>,
        type_info: ReflectTypeInfo,
        serialization: ReflectSerializationStrategy,
    ) -> Self {
        let serializable = !matches!(serialization, ReflectSerializationStrategy::None);
        Self {
            type_path,
            display_name: display_name.into(),
            documentation: None,
            type_info,
            serialization,
            role: ReflectTypeRole::Value,
            serializable,
            editor_visible: true,
            remote_visible: false,
            script_visibility: ReflectScriptVisibility::Private,
        }
    }

    /// 将角色设为 Component；World 注册时仍须提供对应组件适配器。
    pub fn as_component(mut self) -> Self {
        self.role = ReflectTypeRole::Component;
        self
    }

    /// 附加类型说明。
    pub fn with_documentation(mut self, documentation: impl Into<String>) -> Self {
        self.documentation = Some(documentation.into());
        self
    }

    /// 将角色设为 Resource；World 注册时仍须提供对应资源适配器。
    pub fn as_resource(mut self) -> Self {
        self.role = ReflectTypeRole::Resource;
        self
    }

    /// 判断当前声明角色是否为 Component。
    pub fn is_component(&self) -> bool {
        self.role == ReflectTypeRole::Component
    }

    /// 判断当前声明角色是否为 Resource。
    pub fn is_resource(&self) -> bool {
        self.role == ReflectTypeRole::Resource
    }

    /// 若类型路径含插件 ID，则表示此注册声明为插件归属；这不单独授权注册或替换。
    pub fn is_plugin_owned(&self) -> bool {
        self.type_path.plugin_id().is_some()
    }

    /// 显式覆盖序列化参与标记；不会自动改写 `serialization` 策略字段。
    pub fn with_serializable(mut self, serializable: bool) -> Self {
        self.serializable = serializable;
        self
    }

    /// 设置 Editor schema 查询可见性。
    pub fn with_editor_visible(mut self, editor_visible: bool) -> Self {
        self.editor_visible = editor_visible;
        self
    }

    /// 设置远程 schema 查询可见性。
    pub fn with_remote_visible(mut self, remote_visible: bool) -> Self {
        self.remote_visible = remote_visible;
        self
    }

    /// 设置脚本宿主投影可见性；与 Editor 和远程可见性独立。
    pub fn with_script_visibility(mut self, script_visibility: ReflectScriptVisibility) -> Self {
        self.script_visibility = script_visibility;
        self
    }

    /// 校验并附加插件归属键；类型路径前缀的注册约束由 VM 注册路径继续验证。
    pub fn with_plugin_id(mut self, plugin_id: impl Into<String>) -> Result<Self, ReflectError> {
        self.type_path = self.type_path.with_plugin_id(plugin_id)?;
        Ok(self)
    }
}
