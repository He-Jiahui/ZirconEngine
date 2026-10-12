//! 声明输入模块的注册身份、平台依赖和三个服务工厂。
//!
//! # 使用时机与职责
//! 宿主在 Core 注册之前调用构造入口；返回的声明使用 Services 初始化层级，要求已注册的 PlatformModule 依赖。
//! 驱动、输入管理器和动作管理器均标记为 Immediate，且各自的服务依赖列表为空；模块依赖与服务依赖是两层不同的声明。
//!
//! # 约束与生命周期
//! 生成描述符与包装闭包均不调用工厂。Core 激活时先冻结并验证模块图，再按冻结计划解析即时服务；直接解析尚未激活模块的服务也可进入该激活路径。
//! 动作启用选项只决定新管理器初始表，三个服务始终被声明。动作配置按值捕获在可重复调用的工厂中；每次实际调用创建新的管理器，而 Core 对已初始化条目返回缓存实例。
//! 描述符克隆共享工厂 Arc；实例是否可调用和何时卸载由 Core 注册代际及调用门禁管理。
//!
//! # 参考
//! 参见 [`module_descriptor_with_config`]、[`InputConfig`]、[`crate::engine_module::factory`] 与 [`RegisteredManagerService`]。
use std::sync::Arc;

use crate::core::framework::input::INPUT_MODULE_NAME;
use crate::core::framework::input::{InputActionManager, InputManager};
use crate::core::framework::platform::PLATFORM_MODULE_NAME;
use crate::core::manager::RegisteredManagerService;
use crate::core::runtime::ServiceObject;
use crate::core::{
    DriverDescriptor, InitLevel, ManagerDescriptor, ModuleDependencySpec, ModuleDescriptor,
    ServiceKind, StartupMode,
};
use crate::engine_module::{factory, qualified_name};

use super::super::runtime::{DefaultInputManager, InputDriver};
use super::InputConfig;

/// 输入驱动的完整静态服务注册名。
///
/// # 使用时机与约束
/// 值为 `InputModule.Driver.InputDriver`，与描述符使用模块名、Driver 种类和 InputDriver 服务名组合出的键一致。
/// 名称用于查询注册身份；常量本身不证明该服务存在、已经创建或可调用。
///
/// # 生命周期
/// 字符串具有静态生命周期，可长期保存名称；使用服务仍须经所属 Core 的注册与解析规则。
///
/// # 参考
/// 参见 [`InputDriver`]、[`module_descriptor_with_config`] 与 [`crate::engine_module::qualified_name`]。
pub const INPUT_DRIVER_NAME: &str = "InputModule.Driver.InputDriver";
/// 输入状态管理器的完整静态服务注册名，取自 Core 的同名常量。
///
/// # 使用时机与约束
/// 值为 `InputModule.Manager.InputManager`；描述符与 resolver 宏按同一注册键配对 InputManager 接口。
/// 这是字符串值的别名，不能替代带运行时来源和代际的管理器句柄。
///
/// # 生命周期
/// 名称具有静态生命周期；实际管理器由注册服务包装持有，消费者使用所属 Core 重新解析身份后取得调用守卫。
///
/// # 参考
/// 参见 [`InputManager`]、[`RegisteredManagerService`] 与 [`module_descriptor_with_config`]。
pub const INPUT_MANAGER_NAME: &str = crate::core::manager::INPUT_MANAGER_NAME;
/// 命名动作管理器的完整静态服务注册名，取自 Core 的同名常量。
///
/// # 使用时机与约束
/// 值为 `InputModule.Manager.InputActionManager`；resolver 宏按此键请求 InputActionManager 接口。
/// 此服务与物理输入状态管理器使用不同注册身份；启用选项为 false 时仍声明此服务。
///
/// # 生命周期
/// 字符串可长期保存；管理器句柄与调用守卫的有效期由其原 Core 校验，名称不会延长运行时或实例的生命。
///
/// # 参考
/// 参见 [`InputActionManager`]、[`RegisteredManagerService`] 与 [`module_descriptor_with_config`]。
pub const INPUT_ACTION_MANAGER_NAME: &str = crate::core::manager::INPUT_ACTION_MANAGER_NAME;

/// 生成使用默认 InputConfig 的输入模块描述符。
///
/// # 返回
/// 返回拥有模块元数据、平台依赖及三个即时服务工厂的 [`ModuleDescriptor`]；默认配置禁用初始动作且保存空表。
///
/// # 使用时机与生命周期
/// 用于默认 InputModule 入口的注册准备，每次调用生成新声明和工厂；驱动与两个管理器仍会被声明。
/// 返回描述符拥有捕获的默认配置，调用完成后无需保存临时配置。实际注册、激活与工厂调用由 Core 后续推进。
///
/// # 参考
/// 参见 [`InputConfig::default`]、[`module_descriptor_with_config`] 与 [`crate::engine_module::EngineModule`]。
pub fn module_descriptor() -> ModuleDescriptor {
    module_descriptor_with_config(InputConfig::default())
}

/// 在 CoreRuntime 注册阶段固定平台依赖与服务工厂；动作配置移动进工厂，供服务启动时构造求值器。
///
/// 消费启动配置并生成输入模块及服务的注册声明。
///
/// # 参数
/// `config` 按值移入动作管理器工厂，决定该工厂以后创建管理器时采用的初始有效动作表。
/// 字段 enabled 不控制是否声明驱动、输入管理器或动作管理器；原表也不在此处验证或求值。
///
/// # 返回
/// 返回 Services 层级的 [`ModuleDescriptor`]，声明 PlatformModule 模块依赖、一个驱动和两个管理器；三个服务均为 Immediate，各自服务依赖列表均为空。
/// 描述符使用默认 NoopModuleLifecycle；驱动工厂直接返回 InputDriver，管理器工厂返回以框架 trait 类型参数包装的 RegisteredManagerService。
///
/// # 使用时机与约束
/// 在 Core 模块图冻结前提供给 register_module；首次激活的冻结图要求平台依赖已注册。生成描述符不完成注册、不激活依赖，也不调用三个工厂。
/// 即时服务的实际启动次序由 Core 冻结计划决定，不能把本地构建链的声明次序当作所有调用路径上的执行保证。
///
/// # 生命周期
/// 返回描述符的工厂 Arc 持有移入配置；克隆描述符会共享该工厂与其捕获配置。每次真正调用工厂都创建新实例，Core 对已初始化服务条目再解析时复用缓存实例。
/// 动作管理器取得独立有效表；工厂外其它配置副本的改写和后续配置变量的生命期均不影响它。运行期重绑定应使用 InputActionManager.set_action_map。
///
/// # 参考
/// 参见 [`InputConfig::action_manager`]、[`crate::engine_module::factory`]、[`RegisteredManagerService`] 与 [`ModuleDescriptor`]。
pub fn module_descriptor_with_config(config: InputConfig) -> ModuleDescriptor {
    let action_config = config;
    ModuleDescriptor::new(
        INPUT_MODULE_NAME,
        "High-level input routing and action maps",
    )
    .with_init_level(InitLevel::Services)
    .with_module_dependency(ModuleDependencySpec::named(PLATFORM_MODULE_NAME))
    .with_driver(DriverDescriptor::new(
        qualified_name(INPUT_MODULE_NAME, ServiceKind::Driver, "InputDriver"),
        StartupMode::Immediate,
        Vec::new(),
        // 在 Core 请求初始化输入驱动时创建新的驱动服务对象。
        //
        // # 参数
        // `_` 是调用方传入并被忽略的 `&CoreWeak`；闭包不升级、解析或保存这个弱句柄，也没有捕获配置。
        //
        // # 返回
        // 返回 `Ok(Arc::new(InputDriver) as ServiceObject)`；InputDriver 为零字段身份值，本闭包没有主动返回 CoreError 的分支。
        //
        // # 使用时机与生命周期
        // factory 只把此闭包装入共享的 Fn；每次实际调用分配新的 Arc 服务对象，返回对象不借用弱句柄。
        // Core 保存服务对象并控制调用与卸载；创建标记值不启动平台事件循环或设备轮询。
        //
        // # 参考
        // 参见 [`InputDriver`]、[`ServiceObject`] 与 [`crate::engine_module::factory`]。
        factory(|_| Ok(Arc::new(InputDriver) as ServiceObject)),
    ))
    .with_manager(ManagerDescriptor::new(
        qualified_name(INPUT_MODULE_NAME, ServiceKind::Manager, "InputManager"),
        StartupMode::Immediate,
        Vec::new(),
        // 在初始化物理输入状态服务时构造并包装新的默认输入管理器。
        //
        // # 参数
        // `_` 是临时借用的 `&CoreWeak`，本闭包忽略它，不解析依赖、不保存运行时句柄，也不读取动作启用选项。
        //
        // # 返回
        // 返回新的 DefaultInputManager Arc，装入 `RegisteredManagerService<dyn InputManager>`，再以 ServiceObject 类型擦除后置于 Ok 中。
        // RegisteredManagerService::new 只拥有给定 Arc；这一步不执行注册或取得运行时调用许可。
        //
        // # 使用时机与生命周期
        // 只有 Core 真正调用已保存的 Fn 时才构造管理器；每次调用拥有新的输入状态和互斥锁，外层包装拥有实际管理器 Arc，不借用临时弱句柄。
        // 已初始化条目由 Core 复用；消费者经注册身份解析获取调用守卫，宿主随后明确推进帧并投递事件。
        //
        // # 参考
        // 参见 [`DefaultInputManager`]、[`InputManager`]、[`RegisteredManagerService`] 与 [`ServiceObject`]。
        factory(|_| {
            let manager = Arc::new(DefaultInputManager::default());
            Ok(
                Arc::new(RegisteredManagerService::<dyn InputManager>::new(manager))
                    as ServiceObject,
            )
        }),
    ))
    .with_manager(ManagerDescriptor::new(
        qualified_name(
            INPUT_MODULE_NAME,
            ServiceKind::Manager,
            "InputActionManager",
        ),
        StartupMode::Immediate,
        Vec::new(),
        // 用捕获的启动配置创建并包装新的动作管理器服务。
        //
        // # 参数
        // `_` 是调用方提供但被忽略的 `&CoreWeak`；闭包不升级或保存它。action_config 通过 move 捕获，属于工厂的拥有数据，不是调用参数。
        //
        // # 返回
        // 返回新 DefaultInputActionManager 的 Arc，装入 `RegisteredManagerService<dyn InputActionManager>`，以 ServiceObject 类型擦除后置于 Ok 中。
        // 管理器拥有按 action_config 生成的独立有效表、编译代际与工作区；包装构造本身不注册服务。
        //
        // # 使用时机与约束
        // Core 需要初始化该服务时调用工厂；即使 action_config.enabled 为 false，仍创建并包装空初始映射的动作管理器。
        // 闭包通过配置的只读方法构造对象，不消费捕获配置，因而同一 Fn 可以重复调用；每次调用创建新管理器，实际 Core 条目初始化后通常返回缓存实例。
        //
        // # 生命周期
        // 配置随工厂 Arc 存续；描述符 Clone 共享这份工厂捕获数据，而工厂各次返回的管理器不共享求值器状态。
        // 返回对象不借用配置或 CoreWeak；已创建服务的后续映射替换通过 InputActionManager.set_action_map，外部配置副本不能自动更新它。
        //
        // # 参考
        // 参见 [`InputConfig::action_manager`]、[`RegisteredManagerService`]、[`InputActionManager`] 与 [`crate::engine_module::factory`]。
        factory(move |_| {
            let manager = Arc::new(action_config.action_manager());
            Ok(
                Arc::new(RegisteredManagerService::<dyn InputActionManager>::new(
                    manager,
                )) as ServiceObject,
            )
        }),
    ))
}
