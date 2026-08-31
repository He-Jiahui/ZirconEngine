/// Packaging form expected from a product target rule.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProductArtifactKind {
    NativeExecutable,
    MobileApplication,
    BrowserModule,
    Library,
}

impl ProductArtifactKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NativeExecutable => "native_executable",
            Self::MobileApplication => "mobile_application",
            Self::BrowserModule => "browser_module",
            Self::Library => "library",
        }
    }
}

/// Current delivery maturity without implying that a build receipt exists.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProductArtifactDeliveryStatus {
    Runnable,
    Preview,
    ConfigurationOnly,
    Unavailable,
}

impl ProductArtifactDeliveryStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Runnable => "runnable",
            Self::Preview => "preview",
            Self::ConfigurationOnly => "configuration_only",
            Self::Unavailable => "unavailable",
        }
    }

    pub const fn has_configuration_owner(self) -> bool {
        !matches!(self, Self::Unavailable)
    }

    pub const fn has_runnable_artifact(self) -> bool {
        matches!(self, Self::Runnable | Self::Preview)
    }
}

/// Platform-neutral target declaration for a product artifact.
///
/// This is the App equivalent of build target rules, not evidence that a file was built. Exact
/// files, hashes, and runtime ABI identity remain owned by build receipts and Runtime BuildSet
/// manifests.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProductArtifactManifest {
    target_name: &'static str,
    kind: ProductArtifactKind,
    required_build_feature: Option<&'static str>,
    delivery_status: ProductArtifactDeliveryStatus,
}

impl ProductArtifactManifest {
    pub(super) const fn new(
        target_name: &'static str,
        kind: ProductArtifactKind,
        required_build_feature: Option<&'static str>,
        delivery_status: ProductArtifactDeliveryStatus,
    ) -> Self {
        Self {
            target_name,
            kind,
            required_build_feature,
            delivery_status,
        }
    }

    pub const fn target_name(self) -> &'static str {
        self.target_name
    }

    pub const fn kind(self) -> ProductArtifactKind {
        self.kind
    }

    pub const fn required_build_feature(self) -> Option<&'static str> {
        self.required_build_feature
    }

    pub const fn delivery_status(self) -> ProductArtifactDeliveryStatus {
        self.delivery_status
    }

    pub const fn has_configuration_owner(self) -> bool {
        self.delivery_status.has_configuration_owner()
    }

    pub const fn has_runnable_artifact(self) -> bool {
        self.delivery_status.has_runnable_artifact()
    }
}
