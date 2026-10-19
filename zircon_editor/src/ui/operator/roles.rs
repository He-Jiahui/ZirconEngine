//! Validate generated role sidecars before a template can bind editor behavior.
use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::core::commands::EditorCommandRegistry;
use crate::core::editor_operation::EditorOperationPath;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(transparent)]
pub struct OperatorRole(String);

impl OperatorRole {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoleKind {
    Command,
    Slot,
    Region,
    Menu,
    Dock,
    Status,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RoleDefinition {
    pub kind: RoleKind,
    pub screen: String,
    pub required: bool,
    pub command: Option<String>,
    pub slot: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OperatorRoleContract {
    schema_version: u32,
    roles: BTreeMap<OperatorRole, RoleDefinition>,
}

#[derive(Debug, thiserror::Error)]
#[error("operator role contract: {0}")]
pub struct RoleContractError(pub String);

impl OperatorRoleContract {
    pub fn builtin() -> Result<Self, RoleContractError> {
        Self::from_toml(include_str!("../../../assets/ui/operator/contract/roles.contract.toml"))
    }

    pub fn from_toml(source: &str) -> Result<Self, RoleContractError> {
        let contract: Self = toml::from_str(source).map_err(|error| RoleContractError(error.to_string()))?;
        if contract.schema_version != 1 || contract.roles.is_empty() {
            return Err(RoleContractError("expected version 1 and nonempty roles".into()));
        }
        for (role, definition) in &contract.roles {
            let parts: Vec<_> = role.as_str().split('.').collect();
            if parts.len() < 3 || parts[0] != definition.screen || parts.iter().any(|part| {
                !part.bytes().next().is_some_and(|c| c.is_ascii_lowercase())
                    || !part.bytes().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'_')
            }) {
                return Err(RoleContractError(format!("invalid role {} for screen {}", role.as_str(), definition.screen)));
            }
            if (definition.kind == RoleKind::Command) != definition.command.is_some()
                || (definition.kind == RoleKind::Slot) != definition.slot.is_some()
                || definition.command.as_deref() == Some("")
                || definition.slot.as_deref() == Some("")
            {
                return Err(RoleContractError(format!("{} has an inconsistent command/slot kind", role.as_str())));
            }
        }
        Ok(contract)
    }

    pub fn role(&self, id: &str) -> Option<(&OperatorRole, &RoleDefinition)> {
        self.roles.iter().find(|(role, _)| role.as_str() == id)
    }

    pub fn validate_commands(&self, commands: &EditorCommandRegistry) -> Result<(), RoleContractError> {
        for definition in self.roles.values() {
            if let Some(command) = &definition.command {
                let path = EditorOperationPath::parse(command.clone())
                    .map_err(|error| RoleContractError(error.to_string()))?;
                if commands.command(&path).is_none() {
                    return Err(RoleContractError(format!("unknown command {command}")));
                }
            }
        }
        Ok(())
    }

    pub fn missing_required_roles(&self, screen: &str, bindings: &[OperatorTemplateBinding]) -> Vec<String> {
        self.roles.iter().filter(|(role, definition)| {
            definition.screen == screen && definition.required
                && !bindings.iter().any(|binding| &binding.role == *role)
        }).map(|(role, _)| role.as_str().to_string()).collect()
    }
}

/// Serialized beside the generated .zui. Node identity is opaque to behavior.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OperatorTemplateBinding {
    pub role: OperatorRole,
    pub control_id: String,
    pub command: Option<String>,
    pub slot: Option<String>,
}

#[derive(Debug)]
pub struct OperatorRoleBindings {
    by_role: BTreeMap<OperatorRole, OperatorTemplateBinding>,
}

impl OperatorRoleBindings {
    pub fn validate(
        contract: &OperatorRoleContract,
        commands: &EditorCommandRegistry,
        screen: &str,
        bindings: Vec<OperatorTemplateBinding>,
    ) -> Result<Self, RoleContractError> {
        contract.validate_commands(commands)?;
        if !contract.roles.values().any(|definition| definition.screen == screen) {
            return Err(RoleContractError(format!("unknown screen {screen}")));
        }
        let missing = contract.missing_required_roles(screen, &bindings);
        if !missing.is_empty() {
            return Err(RoleContractError(format!("missing required roles: {}", missing.join(", "))));
        }
        let mut by_role = BTreeMap::new();
        let mut controls = BTreeSet::new();
        for binding in bindings {
            let Some((_, definition)) = contract.role(binding.role.as_str()) else {
                return Err(RoleContractError(format!("unknown role {}", binding.role.as_str())));
            };
            if definition.screen != screen || definition.command != binding.command || definition.slot != binding.slot {
                return Err(RoleContractError(format!("binding disagrees with {}", binding.role.as_str())));
            }
            if binding.control_id.is_empty() || !controls.insert(binding.control_id.clone()) {
                return Err(RoleContractError("empty or duplicate control identity".into()));
            }
            if by_role.insert(binding.role.clone(), binding).is_some() {
                return Err(RoleContractError("duplicate role".into()));
            }
        }
        Ok(Self { by_role })
    }

    pub fn binding(&self, role: &str) -> Option<&OperatorTemplateBinding> {
        self.by_role.iter().find_map(|(id, binding)| (id.as_str() == role).then_some(binding))
    }
}

#[cfg(test)]
#[path = "tests/roles.rs"]
mod tests;
