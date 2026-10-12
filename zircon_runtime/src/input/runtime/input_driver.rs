/// 注册表中的输入驱动身份；平台事件循环由宿主推进，本类型本身不负责设备轮询。
///
/// # 使用时机
/// 用作输入模块描述符中的 Driver 注册身份。通过 [`crate::core::CoreRuntime::register_module`] 注册
/// [`crate::input::module_descriptor`] 返回的描述符；需要按具体类型取得该身份时，可把
/// [`crate::input::INPUT_DRIVER_NAME`] 传给 [`crate::core::CoreRuntime::resolve_driver`] 或
/// [`crate::core::CoreRuntime::resolve_driver_handle`]，并以本类型作为泛型参数。
///
/// # 数据与构造
/// 本类型没有字段；派生的 `Default` 只构造此值，不采样设备或创建平台事件循环。
/// 默认输入模块描述符将模块初始化级别设为 [`crate::core::InitLevel::Services`]，本服务的启动模式设为
/// [`crate::core::StartupMode::Immediate`]。构造描述符只保存工厂闭包，不执行 builder；工厂被调用时才将
/// 默认值放入 `Arc` 并返回服务对象。实际初始化顺序由 Core 服务生命周期决定。
///
/// # 调用与生命周期
/// 直接解析返回 [`crate::core::runtime::ServiceCallGuard`]，可保留解析返回的
/// [`crate::core::runtime::ServiceHandle`] 供后续调用进入；闲置 handle 不持有服务实例。每次
/// [`crate::core::runtime::ServiceHandle::enter`] 重新核对运行时、身份代际和服务可用性并取得调用准入。
/// 调用 guard 在存续期间持有实例与准入，释放 guard 后结束该次调用；保留 handle 不等于保留这次准入。
/// 本类型没有输入轮询方法，设备采样与事件循环仍由宿主推进。
///
/// # 参考
/// 参见 [`crate::input::module_descriptor`]、[`crate::input::INPUT_DRIVER_NAME`]、
/// [`crate::core::CoreRuntime::register_module`]、[`crate::core::CoreRuntime::resolve_driver`]、
/// [`crate::core::CoreRuntime::resolve_driver_handle`] 与 [`crate::core::runtime::ServiceHandle::enter`]。
#[derive(Debug, Default)]
pub struct InputDriver;
