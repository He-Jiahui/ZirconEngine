use zircon_runtime::platform::PlatformTarget;

/// Platform family accepted by an App target rule, independent of its runtime execution mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProductPlatformClass {
    Desktop,
    DesktopOrHeadless,
    Browser,
    Mobile,
    HostProvided,
}

impl ProductPlatformClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Desktop => "desktop",
            Self::DesktopOrHeadless => "desktop_or_headless",
            Self::Browser => "browser",
            Self::Mobile => "mobile",
            Self::HostProvided => "host_provided",
        }
    }

    pub const fn accepts(self, target: PlatformTarget) -> bool {
        match self {
            Self::Desktop => target.is_desktop(),
            Self::DesktopOrHeadless => {
                target.is_desktop() || matches!(target, PlatformTarget::Headless)
            }
            Self::Browser => target.is_browser(),
            Self::Mobile => target.is_mobile(),
            Self::HostProvided => true,
        }
    }
}

/// Admission rule for a host capability; Runtime remains the backend capability authority.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProductCapabilityRequirement {
    Required,
    Optional,
    Forbidden,
    HostProvided,
}

impl ProductCapabilityRequirement {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Required => "required",
            Self::Optional => "optional",
            Self::Forbidden => "forbidden",
            Self::HostProvided => "host_provided",
        }
    }
}

/// App-owned host requirements applied before querying Runtime's concrete backend matrix.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProductHostCapabilityPolicy {
    platform: ProductPlatformClass,
    window: ProductCapabilityRequirement,
    input: ProductCapabilityRequirement,
    render: ProductCapabilityRequirement,
}

impl ProductHostCapabilityPolicy {
    pub(super) const fn new(
        platform: ProductPlatformClass,
        window: ProductCapabilityRequirement,
        input: ProductCapabilityRequirement,
        render: ProductCapabilityRequirement,
    ) -> Self {
        Self {
            platform,
            window,
            input,
            render,
        }
    }

    pub const fn platform(self) -> ProductPlatformClass {
        self.platform
    }

    pub const fn window(self) -> ProductCapabilityRequirement {
        self.window
    }

    pub const fn input(self) -> ProductCapabilityRequirement {
        self.input
    }

    pub const fn render(self) -> ProductCapabilityRequirement {
        self.render
    }
}
