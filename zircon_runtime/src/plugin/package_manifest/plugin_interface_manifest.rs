//! 声明插件导出的桥接接口及方法 ABI，供包校验和原生桥接绑定构建。
//! 声明不携带可调用实现；实际回调、接口槽和生命周期由注册及加载端提供。
use serde::{Deserialize, Serialize};

use crate::core::framework::script::{ScriptHostParameterDescriptor, ScriptHostValueKind};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 同一包内接口 ID、方法名和槽位需唯一；包校验阶段才会拒绝冲突。
pub struct PluginInterfaceManifest {
    pub id: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub methods: Vec<PluginInterfaceMethodManifest>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 方法槽用于跨 ABI 调用；参数、返回类型和必需能力是宿主暴露的方法契约。
pub struct PluginInterfaceMethodManifest {
    pub name: String,
    pub method_slot: u32,
    #[serde(default = "default_bridge_method_return_value_kind")]
    pub return_value_kind: ScriptHostValueKind,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parameters: Vec<ScriptHostParameterDescriptor>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_capabilities: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub documentation: Option<String>,
}

impl PluginInterfaceManifest {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            methods: Vec::new(),
        }
    }

    pub fn with_method(mut self, method: PluginInterfaceMethodManifest) -> Self {
        self.methods.push(method);
        self
    }

    /// 根据已验证的接口声明查找方法；未校验的重复名称只会返回第一项。
    pub fn method(&self, name: &str) -> Option<&PluginInterfaceMethodManifest> {
        self.methods.iter().find(|method| method.name == name)
    }
}

impl PluginInterfaceMethodManifest {
    pub fn new(name: impl Into<String>, method_slot: u32) -> Self {
        Self {
            name: name.into(),
            method_slot,
            return_value_kind: ScriptHostValueKind::Null,
            parameters: Vec::new(),
            required_capabilities: Vec::new(),
            documentation: None,
        }
    }

    pub fn with_return_value_kind(mut self, return_value_kind: ScriptHostValueKind) -> Self {
        self.return_value_kind = return_value_kind;
        self
    }

    pub fn with_parameter(mut self, parameter: ScriptHostParameterDescriptor) -> Self {
        self.parameters.push(parameter);
        self
    }

    /// 即使公开字段被反序列化或直接改成无序列表，也按名称排序并去重。
    pub fn with_required_capability(mut self, capability: impl Into<String>) -> Self {
        let capability = capability.into();
        if !self
            .required_capabilities
            .iter()
            .any(|candidate| candidate == &capability)
        {
            self.required_capabilities.push(capability);
        }
        self.required_capabilities.sort();
        self
    }

    pub fn with_documentation(mut self, documentation: impl Into<String>) -> Self {
        self.documentation = Some(documentation.into());
        self
    }
}

fn default_bridge_method_return_value_kind() -> ScriptHostValueKind {
    ScriptHostValueKind::Null
}
