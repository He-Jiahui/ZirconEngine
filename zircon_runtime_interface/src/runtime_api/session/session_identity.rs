//! Editor 保留的会话完整身份，区分 Runtime 所属实例、会话和已发布的传输代次。
//! 构造和投影返回描述值；身份比较涵盖全部字段，不能凭裸 ABI 句柄替代。
//! 保存身份只保留项目字符串，不持有 DLL、会话或其资源的销毁权限。

use std::sync::Arc;

use crate::handles::ZrRuntimeSessionHandle;

/// Immutable owner-qualified identity for one editor-visible runtime transport.
///
/// App creates the base descriptor when it creates a runtime session. Consumers must retain the
/// complete value with any opaque runtime handle; a raw ABI handle has no meaning on its own.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
/// 组合 Runtime 所属身份与 Editor 投影标签的完整值，供来源检查和资源命名。
///
/// # 使用时机与生命周期
/// App 创建会话后构造基础身份，Editor 发布 gateway 时补入 generation 与 Play 实例。
/// 事件来源、异步操作及预览资源保留完整值，与当前已发布身份比较后再使用。
/// `Clone` 共享项目字符串；它不会延长底层会话或函数表的有效期。
///
/// # 约束
/// 所有字段参与 `Eq` 和 `Hash`。调用方更换任一描述字段时，须同步更换随后保存和比较的身份。
/// 本类型不验证句柄、路径或计数器，也不自动递增 transport epoch。
///
/// # 参考
/// `RuntimeSession::create_with_profile_and_project`、`GatewayGeneration::new`，
/// `EditorRuntimeGatewayHandle::detach_at_identity` 与 `PlayPreviewFrameIdentity::resource_scope`。
pub struct GatewaySessionIdentity {
    /// 所属 Runtime 实例的宿主标识；与会话句柄共同限定其来源。
    runtime_instance: u64,
    /// 该实例内的 ABI 会话句柄副本；单独使用不能区分其他实例复用的数值。
    runtime_session: ZrRuntimeSessionHandle,
    /// Editor 发布 gateway 时补入的代次，参与旧来源拒绝和资源命名。
    gateway_generation: u64,
    /// 创建方给出的传输 epoch；当前 App 创建路径传入 1，描述值自身不更新它。
    transport_epoch: u64,
    /// 可选项目标签的共享字符串；不验证路径，也不持有项目或运行时资源。
    project: Option<Arc<str>>,
    /// Editor 发布时补入的可选 Play 实例，区分所投影的 Play 所有者。
    play_instance: Option<u64>,
}

impl GatewaySessionIdentity {
    /// Constructs the App-owned base identity for one successfully created runtime session.
    /// 构造供 App 保存的基础身份，再由 Editor 发布路径补入投影标签。
    ///
    /// # 参数
    /// - `runtime_instance`、`runtime_session`：所属 Runtime 与成功创建的会话标识。
    /// - `transport_epoch`：创建方选择的传输代次，本方法不分配或递增它。
    /// - `project`：可选项目标签；转移 `Arc` 后只共享字符串的所有权。
    ///
    /// # 返回值与约束
    /// gateway generation 为零、Play 实例为空的基础身份，不验证任何输入。
    /// 调用方须把完整值与该来源的句柄一起保存，并独立维持会话的实际存活期。
    ///
    /// # 参考
    /// `RuntimeSession::create_with_profile_and_project`、[`Self::with_gateway_generation`]。
    pub fn new(
        runtime_instance: u64,
        runtime_session: ZrRuntimeSessionHandle,
        transport_epoch: u64,
        project: Option<Arc<str>>,
    ) -> Self {
        Self {
            runtime_instance,
            runtime_session,
            gateway_generation: 0,
            transport_epoch,
            project,
            play_instance: None,
        }
    }

    /// Describes an editor transport that is intentionally detached from every runtime session.
    /// 构造无 Runtime 会话端点的描述值，供脱离状态的 gateway 使用。
    ///
    /// # 返回值与约束
    /// 返回计数标识为零、会话句柄无效、项目和 Play 标签为空的脱离描述。
    /// 这里只表达无 Runtime 会话的端点状态，不执行解绑、销毁或资源释放。
    ///
    /// # 参考
    /// `DetachedEditorRuntimeGateway::session_identity` 与 `EditorRuntimeGateway::session_identity`。
    pub const fn detached() -> Self {
        Self {
            runtime_instance: 0,
            runtime_session: ZrRuntimeSessionHandle::invalid(),
            gateway_generation: 0,
            transport_epoch: 0,
            project: None,
            play_instance: None,
        }
    }

    pub const fn runtime_instance(&self) -> u64 {
        self.runtime_instance
    }

    pub const fn runtime_session(&self) -> ZrRuntimeSessionHandle {
        self.runtime_session
    }

    pub const fn gateway_generation(&self) -> u64 {
        self.gateway_generation
    }

    pub const fn transport_epoch(&self) -> u64 {
        self.transport_epoch
    }

    /// 借用项目标签文本；返回的字符串引用不能超过本次身份借用的有效期。
    ///
    /// # 参考
    /// `PlayPreviewFrameIdentity::resource_scope`；标签为空与非空都参与完整身份比较。
    pub fn project(&self) -> Option<&str> {
        self.project.as_deref()
    }

    pub const fn play_instance(&self) -> Option<u64> {
        self.play_instance
    }

    /// Adds the generation assigned when an Editor gateway publishes this transport.
    /// 为当前值写入发布方指定的 gateway 代次，并返回更新后的完整身份。
    ///
    /// # 参数与返回值
    /// `gateway_generation` 是 Editor 发布方选定的代次；消耗当前值并返回已更新的身份。
    ///
    /// # 约束
    /// 不分配发布代次或操作底层 gateway。返回值改变了完整身份，调用方须同步采用新值，
    /// 避免把旧身份保存为该次发布的操作、事件或预览来源。
    ///
    /// # 参考
    /// `GatewayGeneration::new`、`EditorRuntimeGatewayHandle::replace_for_play`。
    pub fn with_gateway_generation(mut self, gateway_generation: u64) -> Self {
        self.gateway_generation = gateway_generation;
        self
    }

    /// Adds the Editor Play-in-Editor instance that owns this published transport.
    /// 为当前值写入发布方指定的 Play 所有者标签，并返回更新后的完整身份。
    ///
    /// # 参数与返回值
    /// `play_instance` 是发布方给出的 Play 所有者标签；`None` 表示无该标签。
    /// 消耗当前值并返回已更新的身份，不创建或销毁 Play 会话。
    ///
    /// # 约束
    /// 该字段参与完整身份比较；调用方须把返回值同步用于随后保留的来源身份。
    ///
    /// # 参考
    /// `GatewayGeneration::new` 与 `PlayPreviewFrameIdentity::resource_scope`。
    pub fn with_play_instance(mut self, play_instance: Option<u64>) -> Self {
        self.play_instance = play_instance;
        self
    }
}
