#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct ZrRuntimeSessionHandle(pub u64);

#[repr(transparent)]
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct ZrRuntimeViewportHandle(pub u64);

/// The single viewport exposed by the V1 runtime host ABI.
pub const ZIRCON_RUNTIME_DEFAULT_VIEWPORT_HANDLE_V1: ZrRuntimeViewportHandle =
    ZrRuntimeViewportHandle::new(1);

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
