use zircon_plugin_ai_runtime::behavior_tree::{compile_behavior_tree_toml, CompiledBehaviorTree};
use zircon_runtime::core::framework::ai::AiBehaviorTreeDescriptor;

use super::{AiBehaviorTreeEdit, AiBehaviorTreeEditError};

/// A validated source value, with no mutable document, history, file or runtime authority.
#[derive(Clone, Debug, PartialEq)]
pub struct AiBehaviorTreeAsset {
    descriptor: AiBehaviorTreeDescriptor,
    source: String,
    compiled: CompiledBehaviorTree,
}

/// Validated forward and inverse business values for one edit.
///
/// This value owns no document, revision, history, save token or runtime manager.
/// Its source checks do not replace the host's identity/revision CAS (including ABA
/// protection) or allocation admission before preparing the candidate.
#[derive(Clone, Debug, PartialEq)]
pub struct AiBehaviorTreeEditCandidate {
    before: AiBehaviorTreeAsset,
    after: AiBehaviorTreeAsset,
}

impl AiBehaviorTreeEditCandidate {
    pub fn before(&self) -> &AiBehaviorTreeAsset {
        &self.before
    }

    pub fn after(&self) -> &AiBehaviorTreeAsset {
        &self.after
    }

    /// Returns the forward business value only for the complete expected source.
    /// The host must separately check document identity, revision and its permit.
    pub fn forward_from(&self, current: &AiBehaviorTreeAsset) -> Option<&AiBehaviorTreeAsset> {
        (current == &self.before).then_some(&self.after)
    }

    /// Returns the exact validated inverse, including removed nodes and order.
    /// This does not mutate a document or install an editor history entry.
    pub fn inverse_from(&self, current: &AiBehaviorTreeAsset) -> Option<&AiBehaviorTreeAsset> {
        (current == &self.after).then_some(&self.before)
    }
}

impl AiBehaviorTreeAsset {
    pub fn from_toml(source: &str) -> Result<Self, AiBehaviorTreeEditError> {
        let value: toml::Value = toml::from_str(source)?;
        let descriptor: AiBehaviorTreeDescriptor = value.clone().try_into()?;
        let encoded = toml::Value::try_from(&descriptor)?;
        reject_unknown_fields(&value, &encoded, "")?;
        Self::from_descriptor(descriptor)
    }

    pub fn from_descriptor(
        descriptor: AiBehaviorTreeDescriptor,
    ) -> Result<Self, AiBehaviorTreeEditError> {
        let source = toml::to_string(&descriptor)?;
        // The descriptor-only compiler omits manager parameter and child-count validation.
        // Keep the actual asset import validation sequence instead of duplicating its rules.
        let compiled = compile_behavior_tree_toml(&source)?;
        Ok(Self {
            descriptor,
            source,
            compiled,
        })
    }

    pub fn descriptor(&self) -> &AiBehaviorTreeDescriptor {
        &self.descriptor
    }

    /// Canonical v1 TOML; source comments and formatting are not retained.
    pub fn to_toml(&self) -> &str {
        &self.source
    }

    pub fn compiled(&self) -> &CompiledBehaviorTree {
        &self.compiled
    }

    /// Prepares both business directions through the full runtime asset validator.
    /// A no-op creates no candidate; errors leave the original value untouched.
    pub fn prepare_edit(
        &self,
        edit: &AiBehaviorTreeEdit,
    ) -> Result<Option<AiBehaviorTreeEditCandidate>, AiBehaviorTreeEditError> {
        self.edited(edit).map(|after| {
            after.map(|after| AiBehaviorTreeEditCandidate {
                before: self.clone(),
                after,
            })
        })
    }

    /// Builds a candidate only. A caller must admit its allocations and publish it through
    /// the existing document transaction, revision check, history and save lifecycle.
    pub fn edited(
        &self,
        edit: &AiBehaviorTreeEdit,
    ) -> Result<Option<Self>, AiBehaviorTreeEditError> {
        let mut descriptor = self.descriptor.clone();
        edit.apply(&mut descriptor)?;
        if descriptor == self.descriptor {
            return Ok(None);
        }
        Self::from_descriptor(descriptor).map(Some)
    }
}

// Serde permits unknown struct fields. Reject them rather than silently dropping authored
// data; the serialized DTO supplies the actual schema, including nested parameter values.
fn reject_unknown_fields(
    source: &toml::Value,
    encoded: &toml::Value,
    path: &str,
) -> Result<(), AiBehaviorTreeEditError> {
    match (source, encoded) {
        (toml::Value::Table(source), toml::Value::Table(encoded)) => {
            for (key, value) in source {
                let field = if path.is_empty() {
                    key.clone()
                } else {
                    format!("{path}.{key}")
                };
                let known =
                    encoded
                        .get(key)
                        .ok_or_else(|| AiBehaviorTreeEditError::UnknownField {
                            path: field.clone(),
                        })?;
                reject_unknown_fields(value, known, &field)?;
            }
        }
        (toml::Value::Array(source), toml::Value::Array(encoded)) => {
            for (index, (value, known)) in source.iter().zip(encoded).enumerate() {
                reject_unknown_fields(value, known, &format!("{path}[{index}]"))?;
            }
        }
        _ => {}
    }
    Ok(())
}
