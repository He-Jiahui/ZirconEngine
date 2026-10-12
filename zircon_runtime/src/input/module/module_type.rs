use crate::engine_module::{EngineModule, ModuleDescriptor};

use crate::core::framework::input::INPUT_MODULE_NAME;

use super::module_descriptor;

/// 默认输入模块入口；宿主注册并激活其描述符后，通过管理器服务取得输入状态。
///
/// 为内建模块集合提供默认输入模块的零字段入口。
///
/// # 使用时机与约束
/// 宿主将它作为 EngineModule 值保存，取得描述符后交给 Core 注册和激活；模块本身不保存输入配置、运行时句柄或服务实例。
/// 其 descriptor 入口始终使用默认 InputConfig，初始动作表为空且禁用，但仍声明驱动与两个管理器。
/// 需要启动动作配置时使用带配置的描述符构造函数；本类型没有配置字段或配置构建方法。
///
/// # 生命周期
/// Clone、Copy 与 Default 产生相同的零字段入口值，不创建或复制运行期管理器。名称与描述使用静态字符串，每次生成描述符建立独立声明及其工厂。
/// 生成的描述符保存工厂 `Arc`；Core 注册时克隆该 `Arc` 并保存到 `ServiceEntry`，解析时取得工厂克隆。工厂成功返回且身份与生命周期复核通过后，`ServiceEntry` 保存实例的共享所有权；入口值存在本身不证明服务已运行。
///
/// # 参考
/// 参见 [`EngineModule`]、[`module_descriptor`] 与 [`super::module_descriptor_with_config`]。
#[derive(Clone, Copy, Debug, Default)]
pub struct InputModule;

/// 以规范模块身份和默认服务声明实现宿主 EngineModule 接口。
///
/// # 使用时机与约束
/// 宿主收集、排序和注册模块时调用这些方法；模块名、描述与返回的默认描述符保持同一输入模块身份。
/// 接口要求 Send、Sync 与 Debug；这些类型约束不把本实现的声明方法变成服务初始化或运行期同步协议。
///
/// # 生命周期
/// 各方法临时借用零字段入口；名称和描述的本实现返回值是静态字符串，描述符则按值拥有新生成的元数据及工厂。
/// Core 注册、依赖验证、服务创建和激活均由调用方后续推进。
///
/// # 参考
/// 参见 [`EngineModule`]、[`Self::module_name`]、[`Self::module_description`] 与 [`Self::descriptor`]。
impl EngineModule for InputModule {
    /// 返回默认输入模块的规范身份名。
    ///
    /// # 参数
    /// `self` 在调用期间借用模块入口；此零字段实现没有可变配置或运行时状态可读取。
    ///
    /// # 返回
    /// 本实现返回 `INPUT_MODULE_NAME`，即静态字符串 `InputModule`；值与描述符的模块名一致。
    ///
    /// # 使用时机与生命周期
    /// 用于宿主模块索引、排序和诊断；本实现的字符串具有静态生命周期，不依赖入口值的存续。
    /// 通过 EngineModule trait 访问时仍遵循其借用返回签名；此方法不确认该名字已在 Core 注册或激活。
    ///
    /// # 参考
    /// 参见 [`INPUT_MODULE_NAME`]、[`EngineModule`] 与 [`Self::descriptor`]。
    fn module_name(&self) -> &'static str {
        INPUT_MODULE_NAME
    }

    /// 返回默认输入模块的人可读职责描述。
    ///
    /// # 参数
    /// `self` 仅在调用期间借用零字段模块入口，不读取服务状态或动作配置。
    ///
    /// # 返回
    /// 本实现返回静态字符串 `High-level input routing and action maps`，与描述符构造入口的 description 字段内容一致。
    ///
    /// # 使用时机与生命周期
    /// 供模块说明和诊断展示；此字符串是静态值，不借用输入配置或管理器。
    /// 描述文字不是当前动作启用状态、设备可用性或服务激活结果。
    ///
    /// # 参考
    /// 参见 [`EngineModule`]、[`Self::module_name`] 与 [`module_descriptor`]。
    fn module_description(&self) -> &'static str {
        "High-level input routing and action maps"
    }

    /// 为默认输入模块生成一份新的服务描述符。
    ///
    /// # 参数
    /// `self` 在调用期间借用零字段入口；此方法不读取外部或已有的 InputConfig。
    ///
    /// # 返回
    /// 返回 module_descriptor 构造的 [`ModuleDescriptor`]，使用默认禁用动作的配置，并拥有 Services 层级、平台模块依赖与三个即时工厂声明。
    ///
    /// # 使用时机与约束
    /// 用于宿主排序和注册准备；默认动作选项不会省略驱动或两个管理器。需要非默认动作表时应从带配置的构造入口取得描述符。
    /// 调用此方法不注册模块、不验证完整模块图，也不实例化或激活服务；平台注册与 Core 图验证属于后续调用。
    ///
    /// # 生命周期
    /// 每次调用生成新声明及工厂，返回值不借用 self，入口复制或销毁不影响已交出的描述符。
    /// 描述符被 Core 使用后，已创建服务的生命周期由注册表与调用守卫推进。
    ///
    /// # 参考
    /// 参见 [`module_descriptor`]、[`super::module_descriptor_with_config`] 与 [`EngineModule`]。
    fn descriptor(&self) -> ModuleDescriptor {
        module_descriptor()
    }
}
