use serde::{Deserialize, Serialize};
use zircon_runtime_interface::resource::{ResourceLocator, ResourceLocatorError, ResourceScheme};

/// Declares whether an asset type may be authored in the project source.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssetSourceWritePolicy {
    ProjectOnly,
    #[default]
    ReadOnly,
}

/// Canonical editor-side source classification.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssetSourceKind {
    Project,
    Package,
    Builtin,
    Library,
    Derived,
    Transient,
}

impl AssetSourceKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Project => "project",
            Self::Package => "package",
            Self::Builtin => "builtin",
            Self::Library => "library",
            Self::Derived => "derived",
            Self::Transient => "transient",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssetWriteAccess {
    Writable,
    #[default]
    ReadOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetSourceAuthority {
    kind: AssetSourceKind,
    write_access: AssetWriteAccess,
}

impl Default for AssetSourceAuthority {
    fn default() -> Self {
        Self::new(AssetSourceWritePolicy::ReadOnly, AssetSourceKind::Transient)
    }
}

impl AssetSourceAuthority {
    pub fn from_locator(policy: AssetSourceWritePolicy, locator: &ResourceLocator) -> Self {
        let kind = match locator.scheme() {
            ResourceScheme::Res => AssetSourceKind::Project,
            ResourceScheme::Library => AssetSourceKind::Library,
            ResourceScheme::Package => AssetSourceKind::Package,
            ResourceScheme::Builtin => AssetSourceKind::Builtin,
            ResourceScheme::Memory => AssetSourceKind::Transient,
        };
        Self::new(policy, kind)
    }

    pub fn from_locator_str(
        policy: AssetSourceWritePolicy,
        locator: &str,
    ) -> Result<Self, ResourceLocatorError> {
        ResourceLocator::parse(locator).map(|locator| Self::from_locator(policy, &locator))
    }

    /// Resolves an asset or canonical source-root target through `ResourceLocator` validation.
    pub fn from_target_str(
        policy: AssetSourceWritePolicy,
        target: &str,
    ) -> Result<Self, ResourceLocatorError> {
        let fixed_root_kind = match target {
            "res://" => Some(AssetSourceKind::Project),
            "lib://" => Some(AssetSourceKind::Library),
            "builtin://" => Some(AssetSourceKind::Builtin),
            "mem://" => Some(AssetSourceKind::Transient),
            _ => None,
        };
        if let Some(kind) = fixed_root_kind {
            return Ok(Self::new(policy, kind));
        }

        let normalized_root = if let Some(package_id) = target.strip_prefix("package://") {
            (!package_id.is_empty() && !package_id.contains('/'))
                .then(|| format!("{target}/__root__"))
        } else {
            None
        };
        ResourceLocator::parse(normalized_root.as_deref().unwrap_or(target))
            .map(|locator| Self::from_locator(policy, &locator))
    }

    pub fn derived(policy: AssetSourceWritePolicy) -> Self {
        Self::new(policy, AssetSourceKind::Derived)
    }

    pub fn transient(policy: AssetSourceWritePolicy) -> Self {
        Self::new(policy, AssetSourceKind::Transient)
    }

    pub fn kind(self) -> AssetSourceKind {
        self.kind
    }

    pub fn write_access(self) -> AssetWriteAccess {
        self.write_access
    }

    pub fn is_writable(self) -> bool {
        self.write_access == AssetWriteAccess::Writable
    }

    fn new(policy: AssetSourceWritePolicy, kind: AssetSourceKind) -> Self {
        let write_access =
            if policy == AssetSourceWritePolicy::ProjectOnly && kind == AssetSourceKind::Project {
                AssetWriteAccess::Writable
            } else {
                AssetWriteAccess::ReadOnly
            };
        Self { kind, write_access }
    }
}

#[cfg(test)]
#[path = "source_authority/fixed_root_classification_tests.rs"]
mod fixed_root_classification_tests;
