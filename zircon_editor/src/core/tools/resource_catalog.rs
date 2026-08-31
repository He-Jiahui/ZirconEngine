use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::{
    ToolOwnerGeneration, ToolResourceKey, ToolResourceKindId, ToolResourceSet, ToolScopeKind,
};

pub const DEFAULT_MAX_REGISTERED_TOOL_RESOURCE_KINDS: usize = 256;
pub const DEFAULT_MAX_TOOL_RESOURCE_KINDS_PER_OWNER: usize = 64;
const BUILTIN_TOOL_RESOURCE_NAMESPACE: &str = "editor.";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolResourceChannelPolicy {
    Forbidden,
    Optional,
    Required,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ToolResourceKindDeclaration {
    kind: ToolResourceKindId,
    supported_scopes: Box<[ToolScopeKind]>,
    channel_policy: ToolResourceChannelPolicy,
}

impl ToolResourceKindDeclaration {
    pub fn new(
        kind: ToolResourceKindId,
        supported_scopes: impl IntoIterator<Item = ToolScopeKind>,
        channel_policy: ToolResourceChannelPolicy,
    ) -> Result<Self, ToolResourceKindRegistrationError> {
        let mut supported_scopes = supported_scopes.into_iter().collect::<Vec<_>>();
        supported_scopes.sort_unstable();
        supported_scopes.dedup();
        if supported_scopes.is_empty() {
            return Err(ToolResourceKindRegistrationError::EmptySupportedScopes);
        }
        Ok(Self {
            kind,
            supported_scopes: supported_scopes.into_boxed_slice(),
            channel_policy,
        })
    }

    pub fn kind(&self) -> &ToolResourceKindId {
        &self.kind
    }

    pub fn supported_scopes(&self) -> &[ToolScopeKind] {
        &self.supported_scopes
    }

    pub const fn channel_policy(&self) -> ToolResourceChannelPolicy {
        self.channel_policy
    }
}

impl<'de> Deserialize<'de> for ToolResourceKindDeclaration {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct SerializedDeclaration {
            kind: ToolResourceKindId,
            supported_scopes: Box<[ToolScopeKind]>,
            channel_policy: ToolResourceChannelPolicy,
        }

        let value = SerializedDeclaration::deserialize(deserializer)?;
        Self::new(
            value.kind,
            value.supported_scopes.into_vec(),
            value.channel_policy,
        )
        .map_err(serde::de::Error::custom)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ToolResourceKindRegistration {
    kind: ToolResourceKindId,
    owner_generation: ToolOwnerGeneration,
    supported_scopes: Box<[ToolScopeKind]>,
    channel_policy: ToolResourceChannelPolicy,
}

impl ToolResourceKindRegistration {
    pub(crate) fn from_declaration(
        declaration: ToolResourceKindDeclaration,
        owner_generation: ToolOwnerGeneration,
    ) -> Self {
        Self {
            kind: declaration.kind,
            owner_generation,
            supported_scopes: declaration.supported_scopes,
            channel_policy: declaration.channel_policy,
        }
    }

    fn builtin(kind: ToolResourceKindId, scope: ToolScopeKind) -> Self {
        Self {
            kind,
            owner_generation: ToolOwnerGeneration::BUILTIN,
            supported_scopes: vec![scope].into_boxed_slice(),
            channel_policy: ToolResourceChannelPolicy::Forbidden,
        }
    }

    pub fn kind(&self) -> &ToolResourceKindId {
        &self.kind
    }

    pub const fn owner_generation(&self) -> ToolOwnerGeneration {
        self.owner_generation
    }

    pub fn supported_scopes(&self) -> &[ToolScopeKind] {
        &self.supported_scopes
    }

    pub const fn channel_policy(&self) -> ToolResourceChannelPolicy {
        self.channel_policy
    }

    fn supports_scope(&self, scope: ToolScopeKind) -> bool {
        self.supported_scopes.binary_search(&scope).is_ok()
    }
}

impl<'de> Deserialize<'de> for ToolResourceKindRegistration {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct SerializedRegistration {
            kind: ToolResourceKindId,
            owner_generation: ToolOwnerGeneration,
            supported_scopes: Box<[ToolScopeKind]>,
            channel_policy: ToolResourceChannelPolicy,
        }

        let value = SerializedRegistration::deserialize(deserializer)?;
        let declaration = ToolResourceKindDeclaration::new(
            value.kind,
            value.supported_scopes.into_vec(),
            value.channel_policy,
        )
        .map_err(serde::de::Error::custom)?;
        Ok(Self::from_declaration(declaration, value.owner_generation))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToolResourceKindRegistrationError {
    EmptySupportedScopes,
}

impl std::fmt::Display for ToolResourceKindRegistrationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptySupportedScopes => {
                formatter.write_str("a tool resource kind must support at least one scope")
            }
        }
    }
}

impl std::error::Error for ToolResourceKindRegistrationError {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ToolResourceCatalogError {
    DuplicateKind {
        kind: ToolResourceKindId,
    },
    ReservedBuiltinNamespace {
        kind: ToolResourceKindId,
        owner_generation: ToolOwnerGeneration,
    },
    UnregisteredKind {
        kind: ToolResourceKindId,
    },
    UnsupportedScope {
        kind: ToolResourceKindId,
        actual: ToolScopeKind,
        supported: Box<[ToolScopeKind]>,
    },
    ChannelForbidden {
        kind: ToolResourceKindId,
    },
    ChannelRequired {
        kind: ToolResourceKindId,
    },
    GlobalCapacityReached {
        max_registered_kinds: usize,
    },
    OwnerCapacityReached {
        owner_generation: ToolOwnerGeneration,
        max_registered_kinds: usize,
    },
}

impl std::fmt::Display for ToolResourceCatalogError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateKind { kind } => {
                write!(
                    formatter,
                    "tool resource kind {} is already registered",
                    kind.as_str()
                )
            }
            Self::ReservedBuiltinNamespace {
                kind,
                owner_generation,
            } => write!(
                formatter,
                "tool owner generation {owner_generation} cannot register reserved built-in resource kind {}",
                kind.as_str()
            ),
            Self::UnregisteredKind { kind } => {
                write!(
                    formatter,
                    "tool resource kind {} is not registered",
                    kind.as_str()
                )
            }
            Self::UnsupportedScope {
                kind,
                actual,
                supported,
            } => write!(
                formatter,
                "tool resource kind {} does not support {actual:?} scope; supported scopes are {supported:?}",
                kind.as_str()
            ),
            Self::ChannelForbidden { kind } => write!(
                formatter,
                "tool resource kind {} forbids channels",
                kind.as_str()
            ),
            Self::ChannelRequired { kind } => write!(
                formatter,
                "tool resource kind {} requires a channel",
                kind.as_str()
            ),
            Self::GlobalCapacityReached {
                max_registered_kinds,
            } => write!(
                formatter,
                "tool resource catalog reached its global capacity of {max_registered_kinds} kinds"
            ),
            Self::OwnerCapacityReached {
                owner_generation,
                max_registered_kinds,
            } => write!(
                formatter,
                "tool owner generation {owner_generation} reached its capacity of {max_registered_kinds} resource kinds"
            ),
        }
    }
}

impl std::error::Error for ToolResourceCatalogError {}

#[derive(Clone, Debug)]
pub(crate) struct ToolResourceCatalog {
    registrations: BTreeMap<ToolResourceKindId, ToolResourceKindRegistration>,
    max_registered_kinds: usize,
    max_registered_kinds_per_owner: usize,
}

impl ToolResourceCatalog {
    pub(crate) fn new() -> Self {
        let mut registrations = BTreeMap::new();
        for registration in [
            ToolResourceKindRegistration::builtin(
                ToolResourceKindId::viewport_input(),
                ToolScopeKind::Viewport,
            ),
            ToolResourceKindRegistration::builtin(
                ToolResourceKindId::modal_surface(),
                ToolScopeKind::Window,
            ),
            ToolResourceKindRegistration::builtin(
                ToolResourceKindId::scene_mode_slot(),
                ToolScopeKind::Viewport,
            ),
        ] {
            registrations.insert(registration.kind().clone(), registration);
        }
        Self {
            registrations,
            max_registered_kinds: DEFAULT_MAX_REGISTERED_TOOL_RESOURCE_KINDS,
            max_registered_kinds_per_owner: DEFAULT_MAX_TOOL_RESOURCE_KINDS_PER_OWNER,
        }
    }

    pub(crate) fn register(
        &mut self,
        registration: ToolResourceKindRegistration,
    ) -> Result<(), ToolResourceCatalogError> {
        if self.registrations.contains_key(registration.kind()) {
            return Err(ToolResourceCatalogError::DuplicateKind {
                kind: registration.kind().clone(),
            });
        }
        if registration.owner_generation() != ToolOwnerGeneration::BUILTIN
            && registration
                .kind()
                .as_str()
                .starts_with(BUILTIN_TOOL_RESOURCE_NAMESPACE)
        {
            return Err(ToolResourceCatalogError::ReservedBuiltinNamespace {
                kind: registration.kind().clone(),
                owner_generation: registration.owner_generation(),
            });
        }
        if self.registrations.len() >= self.max_registered_kinds {
            return Err(ToolResourceCatalogError::GlobalCapacityReached {
                max_registered_kinds: self.max_registered_kinds,
            });
        }
        let owner_count = self
            .registrations
            .values()
            .filter(|entry| entry.owner_generation() == registration.owner_generation())
            .count();
        if owner_count >= self.max_registered_kinds_per_owner {
            return Err(ToolResourceCatalogError::OwnerCapacityReached {
                owner_generation: registration.owner_generation(),
                max_registered_kinds: self.max_registered_kinds_per_owner,
            });
        }
        self.registrations
            .insert(registration.kind().clone(), registration);
        Ok(())
    }

    pub(crate) fn validate(
        &self,
        resources: &ToolResourceSet,
    ) -> Result<(), ToolResourceCatalogError> {
        for resource in resources.as_slice() {
            self.validate_key(resource)?;
        }
        Ok(())
    }

    pub(crate) fn registrations(&self) -> impl Iterator<Item = &ToolResourceKindRegistration> {
        self.registrations.values()
    }

    pub(crate) fn kinds_for_owner(
        &self,
        owner_generation: ToolOwnerGeneration,
    ) -> Box<[ToolResourceKindId]> {
        self.registrations
            .values()
            .filter(|entry| entry.owner_generation() == owner_generation)
            .map(|entry| entry.kind().clone())
            .collect()
    }

    pub(crate) fn remove_owner(
        &mut self,
        owner_generation: ToolOwnerGeneration,
    ) -> Box<[ToolResourceKindRegistration]> {
        let kinds = self.kinds_for_owner(owner_generation);
        kinds
            .iter()
            .filter_map(|kind| self.registrations.remove(kind))
            .collect()
    }

    pub(crate) fn remove_extension_registrations(&mut self) {
        self.registrations.retain(|_, registration| {
            registration.owner_generation() == ToolOwnerGeneration::BUILTIN
        });
    }

    fn validate_key(&self, resource: &ToolResourceKey) -> Result<(), ToolResourceCatalogError> {
        let Some(registration) = self.registrations.get(resource.kind()) else {
            return Err(ToolResourceCatalogError::UnregisteredKind {
                kind: resource.kind().clone(),
            });
        };
        let actual = resource.scope().kind();
        if !registration.supports_scope(actual) {
            return Err(ToolResourceCatalogError::UnsupportedScope {
                kind: resource.kind().clone(),
                actual,
                supported: registration.supported_scopes().to_vec().into_boxed_slice(),
            });
        }
        match (registration.channel_policy(), resource.channel()) {
            (ToolResourceChannelPolicy::Forbidden, Some(_)) => {
                Err(ToolResourceCatalogError::ChannelForbidden {
                    kind: resource.kind().clone(),
                })
            }
            (ToolResourceChannelPolicy::Required, None) => {
                Err(ToolResourceCatalogError::ChannelRequired {
                    kind: resource.kind().clone(),
                })
            }
            (ToolResourceChannelPolicy::Forbidden, None)
            | (ToolResourceChannelPolicy::Optional, _)
            | (ToolResourceChannelPolicy::Required, Some(_)) => Ok(()),
        }
    }
}

impl Default for ToolResourceCatalog {
    fn default() -> Self {
        Self::new()
    }
}
