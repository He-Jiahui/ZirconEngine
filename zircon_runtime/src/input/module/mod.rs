//! 集中输入模块的启动配置、服务描述符与默认模块入口。
//!
//! # 职责与使用时机
//! 私有 `config` 子模块保存配置和值构造入口；`descriptor` 子模块声明服务键和工厂；`module_type` 子模块提供默认 [`InputModule`]。
//! 调用方从 `crate::input` 取得描述符构造入口；`crate::input::prelude` 另提供常用配置、模块类型和服务名。注册描述符后，宿主可显式激活；按需解析服务也会进入所属模块的激活流程。
//!
//! # 约束与生命周期
//! 这些导出保持原类型与函数身份，不复制服务实例。配置决定新建动作服务的初始映射；描述符拥有元数据与工厂，生成它不会创建或启动服务。
//! 默认模块入口使用禁用动作映射的配置，驱动和两个管理器仍被声明；带配置的入口接受独立配置值。
//!
//! # 参考
//! 参见 [`InputConfig`]、[`module_descriptor_with_config`]、[`InputModule`] 与 [`crate::engine_module::EngineModule`]。
#[path = "config.rs"]
mod config;
#[path = "descriptor.rs"]
mod descriptor;
#[path = "module_type.rs"]
mod module_type;

/// 公开输入服务启动配置的原类型。
///
/// # 使用时机与生命周期
/// 用于保存、序列化和传入带配置的描述符入口；重导出不构造配置值或管理器。
/// 配置拥有原动作表，构造求值器或动作服务时取得独立的有效表，之后修改配置不会回写已经创建的对象。
///
/// # 参考
/// 参见 [`InputConfig`] 与 [`module_descriptor_with_config`]。
pub use config::InputConfig;
/// 公开默认及带配置的描述符构造入口，并提供其三个完整服务注册名。
///
/// # 使用时机与约束
/// 构造入口用于注册之前；完整注册名供按名查询，管理器名与 Core 的 resolver 使用同一值。
/// `INPUT_DRIVER_NAME`、`INPUT_MANAGER_NAME` 与 `INPUT_ACTION_MANAGER_NAME` 描述不同的服务身份。
///
/// # 生命周期
/// 名称是静态字符串；返回的描述符拥有声明和共享工厂。重导出与生成描述符均不代表服务已注册或激活。
///
/// # 参考
/// 参见 [`module_descriptor`]、[`module_descriptor_with_config`]、[`INPUT_DRIVER_NAME`]、[`INPUT_MANAGER_NAME`] 与 [`INPUT_ACTION_MANAGER_NAME`]。
pub use descriptor::{
    module_descriptor, module_descriptor_with_config, INPUT_ACTION_MANAGER_NAME, INPUT_DRIVER_NAME,
    INPUT_MANAGER_NAME,
};
/// 公开内建输入模块的零字段默认入口。
///
/// # 使用时机与生命周期
/// 宿主可将此入口放入 EngineModule 集合，再取得描述符注册；它本身不持有配置或服务状态。
/// 每次调用其 descriptor 方法都生成默认配置的声明；入口的复制和克隆不共享或创建管理器实例。
///
/// # 参考
/// 参见 [`InputModule`]、[`module_descriptor`] 与 [`crate::engine_module::EngineModule`]。
pub use module_type::InputModule;
