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
pub const ZIRCON_RUNTIME_DEFAULT_VIEWPORT_HANDLE_V1: ZrRuntimeViewportHandle =
    ZrRuntimeViewportHandle::new(1);

/// 插件回调上下文的不透明身份；只有发放它的 Host 注册表能解释数值。
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct ZrRuntimePluginHandle(pub u64);

impl ZrRuntimeSessionHandle {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn invalid() -> Self {
        Self(0)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }

    pub const fn is_valid(self) -> bool {
        self.0 != 0
    }
}

impl ZrRuntimeAllocationId {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn invalid() -> Self {
        Self(0)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }

    pub const fn is_valid(self) -> bool {
        self.0 != 0
    }
}

impl ZrRuntimeViewportHandle {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn invalid() -> Self {
        Self(0)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }

    pub const fn is_valid(self) -> bool {
        self.0 != 0
    }
}

impl ZrRuntimePluginHandle {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn invalid() -> Self {
        Self(0)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }

    pub const fn is_valid(self) -> bool {
        self.0 != 0
    }
}
