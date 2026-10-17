//! Runtime 注册候选把中立反射元数据与 World 操作表配对。
//! 候选由工厂或调用方构造，交给 `TypeRegistry` 准入和发布；本对象不拥有 World 实例。

use std::fmt;

use zircon_runtime_interface::reflect::ReflectTypeRegistration;

use super::{ReflectComponent, ReflectResource};

#[derive(Clone)]
/// 反射元数据与可选 World 适配器的一次注册单元；注册表校验后再向编辑器发布模式目录。
///
/// # 约束与生命周期
///
/// 目录发布由 `TypeRegistry` 更新 Runtime 本地 catalog，对外模式读取见 `WorldReflection`。
/// 构造候选与发布目录是两个步骤。`Clone` 复制元数据和适配器表，不复制 World 中的组件或资源。
/// 适配器的路径及字段槽位须与对应元数据配对，实时访问门面再检查角色与能力是否存在。
///
/// # 参考
///
/// `TypeRegistry::register`、`WorldReflection`。
pub struct RuntimeTypeRegistration {
    /// 候选的拥有型元数据；注册表接纳后以 catalog 中规范化的版本保存。
    pub registration: ReflectTypeRegistration,
    /// 可选组件访问能力；仅注册元数据时可缺省，组件访问和暂存入口会另行检查。
    pub component: Option<ReflectComponent>,
    /// 可选资源访问能力；复制该函数表不复制资源实例。
    pub resource: Option<ReflectResource>,
}

impl RuntimeTypeRegistration {
    /// 建立两类 World 适配器均缺省的元数据候选。
    ///
    /// `registration` 的所有权移入返回对象；这里不执行目录准入或发布，也不创建实例。
    /// 后续调用 `TypeRegistry::register`；实时访问或预检暂存仍须具备相应适配器。
    pub fn metadata(registration: ReflectTypeRegistration) -> Self {
        Self {
            registration,
            component: None,
            resource: None,
        }
    }
}

impl fmt::Debug for RuntimeTypeRegistration {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RuntimeTypeRegistration")
            .field("registration", &self.registration)
            .field("has_component_adapter", &self.component.is_some())
            .field("has_resource_adapter", &self.resource.is_some())
            .finish()
    }
}

/// 比较元数据以及组件、资源适配器的存在性。
///
/// 不比较函数地址或组件表内的路径。`TypeRegistry::upsert_vm_type` 据此判断同元数据候选是否无需替换；
/// 相等结果不能单独用来确认回调身份或访问计划版本。
impl PartialEq for RuntimeTypeRegistration {
    fn eq(&self, other: &Self) -> bool {
        self.registration == other.registration
            && self.component.is_some() == other.component.is_some()
            && self.resource.is_some() == other.resource.is_some()
    }
}
