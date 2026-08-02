use std::collections::{BTreeMap, BTreeSet};

use serde::Deserialize;

use crate::scene::ecs::{QueryAccess, SceneSystemThreadAffinity, SystemParamAccess};
use crate::scene::World;

pub(in crate::plugin::native_plugin_loader) const NATIVE_SYSTEM_WORKER_SAFE_CAPABILITY: &str =
    "runtime.native.system.worker_safe";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(in crate::plugin::native_plugin_loader) enum NativePluginRegistrationThreadAffinity {
    #[default]
    MainThreadOnly,
    WorkerSafe,
}

impl NativePluginRegistrationThreadAffinity {
    const fn runtime_affinity(self) -> SceneSystemThreadAffinity {
        match self {
            Self::MainThreadOnly => SceneSystemThreadAffinity::MainThreadOnly,
            Self::WorkerSafe => SceneSystemThreadAffinity::WorkerSafe,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(in crate::plugin::native_plugin_loader) enum NativeSystemAccessMode {
    Read,
    Write,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(in crate::plugin::native_plugin_loader) enum NativeSystemAccessDomain {
    Component,
    Resource,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::plugin::native_plugin_loader) struct NativeSystemAccessDeclaration {
    pub(in crate::plugin::native_plugin_loader) mode: NativeSystemAccessMode,
    pub(in crate::plugin::native_plugin_loader) domain: NativeSystemAccessDomain,
    pub(in crate::plugin::native_plugin_loader) stable_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::plugin::native_plugin_loader) struct NativeSystemAccessPlan {
    affinity: SceneSystemThreadAffinity,
    declarations: Vec<NativeSystemAccessDeclaration>,
    conservative_world_access: bool,
}

impl NativeSystemAccessPlan {
    pub(in crate::plugin::native_plugin_loader) fn from_manifest(
        thread_affinity: NativePluginRegistrationThreadAffinity,
        raw_access: &[String],
        capabilities: &[String],
    ) -> Result<Self, NativeSystemAccessContractError> {
        let affinity = thread_affinity.runtime_affinity();
        if raw_access.is_empty() || raw_access == ["write:world"] {
            if affinity == SceneSystemThreadAffinity::WorkerSafe {
                return Err(NativeSystemAccessContractError::WorkerSafeRequiresExplicitAccess);
            }
            return Ok(Self {
                affinity,
                declarations: Vec::new(),
                conservative_world_access: true,
            });
        }
        if raw_access.iter().any(|access| access == "write:world") {
            return Err(NativeSystemAccessContractError::WorldAccessMustBeExclusive);
        }
        if affinity == SceneSystemThreadAffinity::WorkerSafe
            && !capabilities
                .iter()
                .any(|capability| capability == NATIVE_SYSTEM_WORKER_SAFE_CAPABILITY)
        {
            return Err(NativeSystemAccessContractError::MissingWorkerSafeCapability);
        }

        let mut declarations = Vec::with_capacity(raw_access.len());
        let mut declared = BTreeMap::new();
        for raw in raw_access {
            let declaration = parse_access_declaration(raw)?;
            let key = (declaration.domain, declaration.stable_id.clone());
            if let Some(existing) = declared.insert(key, declaration.mode) {
                if existing == declaration.mode {
                    return Err(NativeSystemAccessContractError::DuplicateAccess {
                        declaration: raw.clone(),
                    });
                }
                return Err(NativeSystemAccessContractError::ConflictingAccess {
                    domain: declaration.domain,
                    stable_id: declaration.stable_id,
                });
            }
            declarations.push(declaration);
        }
        declarations.sort_by(|left, right| {
            left.domain
                .cmp(&right.domain)
                .then(left.stable_id.cmp(&right.stable_id))
                .then(left.mode.cmp(&right.mode))
        });
        Ok(Self {
            affinity,
            declarations,
            conservative_world_access: false,
        })
    }

    pub(in crate::plugin::native_plugin_loader) const fn affinity(
        &self,
    ) -> SceneSystemThreadAffinity {
        self.affinity
    }

    pub(in crate::plugin::native_plugin_loader) fn declarations(
        &self,
    ) -> &[NativeSystemAccessDeclaration] {
        &self.declarations
    }

    pub(in crate::plugin::native_plugin_loader) const fn has_conservative_world_access(
        &self,
    ) -> bool {
        self.conservative_world_access
    }

    pub(in crate::plugin::native_plugin_loader) fn compile(
        &self,
        world: &mut World,
    ) -> Result<SystemParamAccess, NativeSystemAccessResolveError> {
        let mut access = SystemParamAccess::default();
        if self.conservative_world_access {
            access.add_conservative_world_access();
            return Ok(access);
        }

        let mut component_access = QueryAccess::default();
        for declaration in &self.declarations {
            match declaration.domain {
                NativeSystemAccessDomain::Component => {
                    let component_id = world
                        .registered_dynamic_component_id(&declaration.stable_id)
                        .ok_or_else(|| NativeSystemAccessResolveError::UnknownComponent {
                            stable_id: declaration.stable_id.clone(),
                        })?;
                    let result = match declaration.mode {
                        NativeSystemAccessMode::Read => component_access.add_read(component_id),
                        NativeSystemAccessMode::Write => component_access.add_write(component_id),
                    };
                    result.map_err(|error| NativeSystemAccessResolveError::ConflictingAccess {
                        stable_id: declaration.stable_id.clone(),
                        message: error.to_string(),
                    })?;
                }
                NativeSystemAccessDomain::Resource => {
                    let resource_id = world.external_resource_id(&declaration.stable_id);
                    let result = match declaration.mode {
                        NativeSystemAccessMode::Read => access.add_resource_read(resource_id),
                        NativeSystemAccessMode::Write => access.add_resource_write(resource_id),
                    };
                    result.map_err(|error| NativeSystemAccessResolveError::ConflictingAccess {
                        stable_id: declaration.stable_id.clone(),
                        message: error.to_string(),
                    })?;
                }
            }
        }
        access
            .add_query_access(&component_access)
            .map_err(|error| NativeSystemAccessResolveError::ConflictingAccess {
                stable_id: "component access set".to_string(),
                message: error.to_string(),
            })?;
        Ok(access)
    }
}

pub(in crate::plugin::native_plugin_loader) struct NativeSystemAccessAuthority {
    plugin_id: String,
    known_component_ids: BTreeSet<String>,
    known_resource_ids: BTreeSet<String>,
    granted_capabilities: BTreeSet<String>,
}

impl NativeSystemAccessAuthority {
    pub(in crate::plugin::native_plugin_loader) fn new(
        plugin_id: impl Into<String>,
        known_component_ids: impl IntoIterator<Item = String>,
        known_resource_ids: impl IntoIterator<Item = String>,
        granted_capabilities: impl IntoIterator<Item = String>,
    ) -> Self {
        Self {
            plugin_id: plugin_id.into(),
            known_component_ids: known_component_ids.into_iter().collect(),
            known_resource_ids: known_resource_ids.into_iter().collect(),
            granted_capabilities: granted_capabilities.into_iter().collect(),
        }
    }

    pub(in crate::plugin::native_plugin_loader) fn authorize(
        &self,
        plan: &NativeSystemAccessPlan,
    ) -> Result<(), NativeSystemAccessAuthorityError> {
        if plan.affinity == SceneSystemThreadAffinity::WorkerSafe
            && !self
                .granted_capabilities
                .contains(NATIVE_SYSTEM_WORKER_SAFE_CAPABILITY)
        {
            return Err(NativeSystemAccessAuthorityError::WorkerSafeCapabilityNotGranted);
        }
        for declaration in &plan.declarations {
            let known = match declaration.domain {
                NativeSystemAccessDomain::Component => {
                    self.known_component_ids.contains(&declaration.stable_id)
                }
                NativeSystemAccessDomain::Resource => {
                    self.known_resource_ids.contains(&declaration.stable_id)
                }
            };
            if !known {
                return Err(NativeSystemAccessAuthorityError::UnknownStableId {
                    domain: declaration.domain,
                    stable_id: declaration.stable_id.clone(),
                });
            }
            if self.owns(&declaration.stable_id) {
                continue;
            }
            let required_capability = declaration.required_capability();
            if !self.granted_capabilities.contains(&required_capability) {
                return Err(NativeSystemAccessAuthorityError::CapabilityNotGranted {
                    stable_id: declaration.stable_id.clone(),
                    required_capability,
                });
            }
        }
        Ok(())
    }

    fn owns(&self, stable_id: &str) -> bool {
        stable_id == self.plugin_id
            || stable_id
                .strip_prefix(&self.plugin_id)
                .is_some_and(|suffix| suffix.starts_with('.'))
    }
}

impl NativeSystemAccessDeclaration {
    fn required_capability(&self) -> String {
        format!(
            "runtime.native.ecs.{}.{}.{}",
            self.domain.label(),
            self.mode.label(),
            self.stable_id
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::plugin::native_plugin_loader) enum NativeSystemAccessContractError {
    InvalidDeclaration {
        declaration: String,
    },
    InvalidStableId {
        stable_id: String,
    },
    DuplicateAccess {
        declaration: String,
    },
    ConflictingAccess {
        domain: NativeSystemAccessDomain,
        stable_id: String,
    },
    WorldAccessMustBeExclusive,
    WorkerSafeRequiresExplicitAccess,
    MissingWorkerSafeCapability,
}

impl std::fmt::Display for NativeSystemAccessContractError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidDeclaration { declaration } => write!(
                formatter,
                "access `{declaration}` must use read|write:component|resource:<stable-id>"
            ),
            Self::InvalidStableId { stable_id } => {
                write!(formatter, "access stable id `{stable_id}` is invalid")
            }
            Self::DuplicateAccess { declaration } => {
                write!(
                    formatter,
                    "access `{declaration}` is declared more than once"
                )
            }
            Self::ConflictingAccess { domain, stable_id } => write!(
                formatter,
                "access declares both read and write for {} `{stable_id}`",
                domain.label()
            ),
            Self::WorldAccessMustBeExclusive => {
                formatter.write_str("wildcard `write:world` access must be the only declaration")
            }
            Self::WorkerSafeRequiresExplicitAccess => formatter.write_str(
                "worker-safe systems require explicit component/resource access declarations",
            ),
            Self::MissingWorkerSafeCapability => write!(
                formatter,
                "worker-safe systems require capability `{NATIVE_SYSTEM_WORKER_SAFE_CAPABILITY}`"
            ),
        }
    }
}

impl std::error::Error for NativeSystemAccessContractError {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::plugin::native_plugin_loader) enum NativeSystemAccessAuthorityError {
    WorkerSafeCapabilityNotGranted,
    UnknownStableId {
        domain: NativeSystemAccessDomain,
        stable_id: String,
    },
    CapabilityNotGranted {
        stable_id: String,
        required_capability: String,
    },
}

impl std::fmt::Display for NativeSystemAccessAuthorityError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::WorkerSafeCapabilityNotGranted => write!(
                formatter,
                "host did not grant `{NATIVE_SYSTEM_WORKER_SAFE_CAPABILITY}`"
            ),
            Self::UnknownStableId { domain, stable_id } => write!(
                formatter,
                "unknown {} access id `{stable_id}`",
                domain.label()
            ),
            Self::CapabilityNotGranted {
                stable_id,
                required_capability,
            } => write!(
                formatter,
                "access to `{stable_id}` requires granted capability `{required_capability}`"
            ),
        }
    }
}

impl std::error::Error for NativeSystemAccessAuthorityError {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::plugin::native_plugin_loader) enum NativeSystemAccessResolveError {
    UnknownComponent { stable_id: String },
    ConflictingAccess { stable_id: String, message: String },
}

impl std::fmt::Display for NativeSystemAccessResolveError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownComponent { stable_id } => {
                write!(
                    formatter,
                    "component access id `{stable_id}` is not installed"
                )
            }
            Self::ConflictingAccess { stable_id, message } => {
                write!(
                    formatter,
                    "access `{stable_id}` conflicts while resolving: {message}"
                )
            }
        }
    }
}

impl std::error::Error for NativeSystemAccessResolveError {}

impl NativeSystemAccessDomain {
    const fn label(self) -> &'static str {
        match self {
            Self::Component => "component",
            Self::Resource => "resource",
        }
    }
}

impl NativeSystemAccessMode {
    const fn label(self) -> &'static str {
        match self {
            Self::Read => "read",
            Self::Write => "write",
        }
    }
}

fn parse_access_declaration(
    declaration: &str,
) -> Result<NativeSystemAccessDeclaration, NativeSystemAccessContractError> {
    let mut fields = declaration.splitn(3, ':');
    let mode = match fields.next() {
        Some("read") => NativeSystemAccessMode::Read,
        Some("write") => NativeSystemAccessMode::Write,
        _ => return invalid_declaration(declaration),
    };
    let domain = match fields.next() {
        Some("component") => NativeSystemAccessDomain::Component,
        Some("resource") => NativeSystemAccessDomain::Resource,
        _ => return invalid_declaration(declaration),
    };
    let Some(stable_id) = fields.next() else {
        return invalid_declaration(declaration);
    };
    if stable_id.is_empty()
        || stable_id.trim() != stable_id
        || stable_id.chars().any(char::is_whitespace)
    {
        return Err(NativeSystemAccessContractError::InvalidStableId {
            stable_id: stable_id.to_string(),
        });
    }
    Ok(NativeSystemAccessDeclaration {
        mode,
        domain,
        stable_id: stable_id.to_string(),
    })
}

fn invalid_declaration<T>(declaration: &str) -> Result<T, NativeSystemAccessContractError> {
    Err(NativeSystemAccessContractError::InvalidDeclaration {
        declaration: declaration.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use crate::core::framework::scene::ComponentTypeDescriptor;

    use super::*;

    #[test]
    fn native_system_access_authority_resolves_known_owned_ids() {
        let plan = NativeSystemAccessPlan::from_manifest(
            NativePluginRegistrationThreadAffinity::WorkerSafe,
            &[
                "read:component:physics.Body".to_string(),
                "write:resource:physics.solver".to_string(),
            ],
            &[NATIVE_SYSTEM_WORKER_SAFE_CAPABILITY.to_string()],
        )
        .unwrap();
        let authority = NativeSystemAccessAuthority::new(
            "physics",
            ["physics.Body".to_string()],
            ["physics.solver".to_string()],
            [NATIVE_SYSTEM_WORKER_SAFE_CAPABILITY.to_string()],
        );
        authority.authorize(&plan).unwrap();
        let mut world = World::empty();
        world
            .register_component_type(ComponentTypeDescriptor::new(
                "physics.Body",
                "physics",
                "Physics Body",
            ))
            .unwrap();

        let access = plan.compile(&mut world).unwrap();

        assert!(!access.has_conservative_world_access());
        assert!(world
            .registered_external_resource_id("physics.solver")
            .is_some());
    }

    #[test]
    fn native_system_access_authority_rejects_foreign_or_ungranted_worker_access() {
        let plan = NativeSystemAccessPlan::from_manifest(
            NativePluginRegistrationThreadAffinity::WorkerSafe,
            &["read:component:render.Visible".to_string()],
            &[NATIVE_SYSTEM_WORKER_SAFE_CAPABILITY.to_string()],
        )
        .unwrap();
        let no_grants =
            NativeSystemAccessAuthority::new("physics", ["render.Visible".to_string()], [], []);
        assert!(matches!(
            no_grants.authorize(&plan),
            Err(NativeSystemAccessAuthorityError::WorkerSafeCapabilityNotGranted)
        ));
        let worker_only = NativeSystemAccessAuthority::new(
            "physics",
            ["render.Visible".to_string()],
            [],
            [NATIVE_SYSTEM_WORKER_SAFE_CAPABILITY.to_string()],
        );
        assert!(matches!(
            worker_only.authorize(&plan),
            Err(NativeSystemAccessAuthorityError::CapabilityNotGranted {
                required_capability,
                ..
            }) if required_capability == "runtime.native.ecs.component.read.render.Visible"
        ));
    }
}
