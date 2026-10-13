//! 同步会话创建的借用配置与跨 DLL 唤醒注册契约。
//! 配置字节只借用于创建调用；唤醒函数和 token 会被复制，并持续用于会话的异步通知。
//! 描述值的形状检查不证明地址可读、回调可调用或相关宿主资源仍存活。

use crate::buffer::ZrByteSlice;
use crate::version::{ZIRCON_RUNTIME_ABI_VERSION_V1, ZIRCON_RUNTIME_ABI_VERSION_V3};

/// Session-scoped host wake callback carried across the runtime DLL boundary.
///
/// `token` is opaque to the runtime. A sink is either fully disabled
/// (`token == 0`, `wake == None`) or fully registered (`token != 0`,
/// `wake == Some`). The callback must return promptly. Synchronous destruction of the same
/// session from inside `wake` is rejected so the callback cannot wait on its own quiescence.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
/// 会话范围的宿主唤醒描述，供 Runtime 复制函数地址与不透明路由标识。
///
/// # 使用时机与生命周期
/// 宿主先登记 token，再把描述值传给创建入口；Runtime 会复制注册并在无会话锁时回调。
/// 宿主须保证函数及其 token 对应状态可被回调线程使用，并使回调及时返回且不跨 C 边界展开。
/// 成功销毁会话会阻止新回调并等待在途回调；失败不能作为宿主释放注册资源的依据。
///
/// # 参考
/// Runtime 侧 `RuntimeWakeRegistration::from_abi`、`RuntimeWakeRegistration::wake`，
/// 宿主侧 `RuntimeSession::try_destroy` 与 [`Self::is_valid`]。
pub struct ZrRuntimeWakeSinkV1 {
    /// 唤醒描述版本；当前配对检查只接受 V1。
    pub abi_version: u32,
    /// 宿主登记的非零路由 token；零值仅与无回调配对，Runtime 不解码其含义。
    pub token: u64,
    /// 宿主函数地址；注册期间须保持可调用，字段非空不构成地址与线程安全证明。
    pub wake: Option<unsafe extern "C" fn(u64)>,
}

impl ZrRuntimeWakeSinkV1 {
    /// 创建 V1 的禁用描述，供不需要异步宿主通知的会话使用。
    ///
    /// # 返回值
    /// 零 token 与空回调的合法配对；不会注销任何已经登记的唤醒对象。
    ///
    /// # 参考
    /// [`ZrRuntimeSessionConfigV3::empty`]、[`Self::is_valid`]。
    pub const fn disabled() -> Self {
        Self {
            abi_version: ZIRCON_RUNTIME_ABI_VERSION_V1,
            token: 0,
            wake: None,
        }
    }

    /// 包装宿主已登记的唤醒路由，供会话创建入口复制。
    ///
    /// # 参数
    /// - `token`：宿主分配的路由标识，合法注册需要非零值。
    /// - `wake`：接收该 token 的 C 回调；其可调用期和线程前提由宿主保证。
    ///
    /// # 返回值与约束
    /// 保存 V1 版本和参数，不登记资源，也不验证 token 非零或函数地址有效。
    /// 回调不得在自身执行期间同步销毁同一会话；实际销毁入口会拒绝该重入。
    ///
    /// # 参考
    /// 宿主 `RuntimeWakeRegistration::sink`、Runtime `RuntimeWakeRegistration::from_abi`。
    pub const fn new(token: u64, wake: unsafe extern "C" fn(u64)) -> Self {
        Self {
            abi_version: ZIRCON_RUNTIME_ABI_VERSION_V1,
            token,
            wake: Some(wake),
        }
    }

    /// 检查版本及禁用／登记字段是否成对出现。
    ///
    /// # 返回值与约束
    /// 仅当 V1 且为零 token／无回调或非零 token／有回调时返回 `true`。
    /// 此检查不会调用函数，也不会验证宿主登记状态或证明函数地址的有效期。
    ///
    /// # 参考
    /// `RuntimeWakeRegistration::from_abi` 与布局、配对契约测试
    /// `runtime_v3_session_config_and_wake_sink_layout_and_validity_are_stable`。
    pub const fn is_valid(self) -> bool {
        self.abi_version == ZIRCON_RUNTIME_ABI_VERSION_V1
            && matches!((self.token, self.wake), (0, None) | (1.., Some(_)))
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
/// 创建会话时一次性传入的配置。字节切片在同步创建调用期间借用宿主内存；
/// 运行时须在返回前解析或复制所需内容，唤醒回调遵循独立的注册与停用生命周期。
///
/// # 使用时机与生命周期
/// 传给 [`crate::runtime_api::ZrRuntimeCreateSessionFnV3`]；所有非空切片须在整个同步调用期间
/// 指向已初始化、可读且不被修改的宿主字节。配置的 `Copy` 不延长这些借用。
/// 创建入口先检查预算，再解析 profile 和项目启动参数；保留解析后的值而非这些裸地址。
///
/// # 参考
/// `RuntimeSession::create_with_profile_and_project`、`RuntimeProjectConfig::from_abi_startup_config`，
/// [`ZrRuntimeWakeSinkV1`] 与 [`Self::is_valid`]。
pub struct ZrRuntimeSessionConfigV3 {
    /// 创建配置版本，当前创建入口要求 V3；与内嵌唤醒描述的 V1 版本独立。
    pub abi_version: u32,
    /// 运行模式名称的借用字节；当前 profile 解析器把空值解释为默认 runtime 模式。
    pub profile: ZrByteSlice,
    /// Physical project-root anchor. All project-local paths are resolved below this root.
    /// 项目根的借用 UTF-8 字节；空值表示未指定项目，由启动解析器处理其余参数约束。
    pub project_root: ZrByteSlice,
    /// Optional project-relative versioned DynamicScene document for Play-in-Editor startup.
    /// 借用的项目相对启动场景路径；非空时必须同时提供可解析的项目根。
    pub play_scene: ZrByteSlice,
    /// Optional logical startup report outlet name.
    /// 借用的逻辑报告出口名称；Runtime 验证名称，输出传输由宿主进程持有。
    pub play_report_pipe: ZrByteSlice,
    /// 独立注册的唤醒描述；它的寿命延续到会话销毁，而非只到配置借用结束。
    pub wake_sink: ZrRuntimeWakeSinkV1,
}

impl ZrRuntimeSessionConfigV3 {
    /// 创建 V3 的默认描述，便于调用者在创建前填写选定字段。
    ///
    /// # 返回值与约束
    /// 所有字节切片为空且唤醒禁用的配置；这里只构造描述值，不创建会话或解析项目。
    /// 是否能成功启动仍取决于创建入口的完整检查和所选运行模式。
    ///
    /// # 参考
    /// [`Self::is_valid`]、[`ZrRuntimeWakeSinkV1::disabled`] 与
    /// `RuntimeDynamicSessionProfile::from_bytes`。
    pub const fn empty() -> Self {
        Self {
            abi_version: ZIRCON_RUNTIME_ABI_VERSION_V3,
            profile: ZrByteSlice::empty(),
            project_root: ZrByteSlice::empty(),
            play_scene: ZrByteSlice::empty(),
            play_report_pipe: ZrByteSlice::empty(),
            wake_sink: ZrRuntimeWakeSinkV1::disabled(),
        }
    }

    /// 检查创建配置版本及内嵌唤醒描述的字段配对。
    ///
    /// # 返回值与约束
    /// V3 且 [`ZrRuntimeWakeSinkV1::is_valid`] 成功时返回 `true`。
    /// 不读取切片或解析内容；字节预算、可读借用、profile 与项目语义仍由调用契约和创建入口负责。
    ///
    /// # 参考
    /// `create_session` 的预算及启动参数接纳路径。
    pub const fn is_valid(self) -> bool {
        self.abi_version == ZIRCON_RUNTIME_ABI_VERSION_V3 && self.wake_sink.is_valid()
    }
}
