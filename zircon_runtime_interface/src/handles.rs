//! Runtime 会话、输出分配、视口与插件上下文的不透明数值身份。
//!
//! 包装值只保留数值和零值约定；注册表查找仍决定身份是否存活以及属于哪个提供方。
//! [`ZrRuntimeAllocationId`] 的释放还须绑定原 [`ZrRuntimeSessionHandle`]。

/// Runtime 发放的会话身份；0 表示无效，销毁前须释放该会话的输出分配。
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct ZrRuntimeSessionHandle(pub u64);

/// Runtime 会话中的输出分配身份；须连同原会话交给 release_allocation。
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct ZrRuntimeAllocationId(pub u64);

/// Host 与 Runtime 路由视口请求时使用的不透明身份；0 表示未绑定视口。
#[repr(transparent)]
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct ZrRuntimeViewportHandle(pub u64);

/// The single viewport exposed by the V1 runtime host ABI.
/// 当前 V1 视口入口使用的唯一视口身份，创建载体本身不会绑定原生表面。
pub const ZIRCON_RUNTIME_DEFAULT_VIEWPORT_HANDLE_V1: ZrRuntimeViewportHandle =
    ZrRuntimeViewportHandle::new(1);

/// 插件回调上下文的不透明身份；只有发放它的 Host 注册表能解释数值。
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct ZrRuntimePluginHandle(pub u64);

impl ZrRuntimeSessionHandle {
    /// 包装参数 `raw`，不创建会话，也不验证它仍在 Runtime 注册表中。
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    /// 返回零值占位，供会话创建尚未成功的输出使用。
    pub const fn invalid() -> Self {
        Self(0)
    }

    /// 取出供会话注册表查找的数值，不改变会话状态。
    pub const fn raw(self) -> u64 {
        self.0
    }

    /// 只检查非零约定；失效的历史会话身份也可能通过此检查。
    pub const fn is_valid(self) -> bool {
        self.0 != 0
    }
}

impl ZrRuntimeAllocationId {
    /// 包装参数 `raw`，不创建分配，也不授予重复释放权。
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    /// 返回没有输出分配的零值身份，供规范空结果使用。
    pub const fn invalid() -> Self {
        Self(0)
    }

    /// 取出交还原 Runtime 分配注册表的数值。
    pub const fn raw(self) -> u64 {
        self.0
    }

    /// 只检查非零约定；分配是否仍存活及所属会话由释放入口核对。
    pub const fn is_valid(self) -> bool {
        self.0 != 0
    }
}

impl ZrRuntimeViewportHandle {
    /// 包装参数 `raw`，不绑定视口表面或验证当前会话是否支持该视口。
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    /// 返回未指定有效视口的零值身份。
    pub const fn invalid() -> Self {
        Self(0)
    }

    /// 取出供视口请求路由与序列化使用的数值。
    pub const fn raw(self) -> u64 {
        self.0
    }

    /// 只检查非零约定；可路由性和表面绑定状态仍由入口验证。
    pub const fn is_valid(self) -> bool {
        self.0 != 0
    }
}

impl ZrRuntimePluginHandle {
    /// 包装参数 `raw`，不创建插件上下文或改变 Host 注册表的代际状态。
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    /// 返回未关联插件上下文的零值身份。
    pub const fn invalid() -> Self {
        Self(0)
    }

    /// 取出交给发放该身份的 Host 上下文注册表的数值。
    pub const fn raw(self) -> u64 {
        self.0
    }

    /// 只检查非零约定；上下文存活性和代际匹配须由 Host 注册表证明。
    pub const fn is_valid(self) -> bool {
        self.0 != 0
    }
}
