use zircon_runtime::core::framework::ai::{
    AiBlackboardKeyDescriptor, AiBlackboardSchemaDescriptor,
};

use super::menu_types::AiBlackboardMenuError as Error;
use super::rename::validate_schema;
use super::{AiBlackboardSchemaInput, AiBlackboardSourceVersion};

/// A validated schema authoring value. It cannot replace a live registered schema,
/// prove input-producer coverage, save a file or authorize a document transaction.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AiBlackboardSchemaEditCandidate {
    source: AiBlackboardSourceVersion,
    before: AiBlackboardSchemaDescriptor,
    after: AiBlackboardSchemaDescriptor,
}

impl AiBlackboardSchemaEditCandidate {
    pub(super) fn create_key(
        input: AiBlackboardSchemaInput<'_>,
        index: usize,
        key: &AiBlackboardKeyDescriptor,
    ) -> Result<Self, Error> {
        validate_schema(input.descriptor).map_err(Error::InvalidSchema)?;
        if index > input.descriptor.keys.len() {
            return Err(Error::KeyIndex {
                index,
                count: input.descriptor.keys.len(),
            });
        }
        let mut after = input.descriptor.clone();
        after.keys.insert(index, key.clone());
        validate_schema(&after).map_err(Error::InvalidSchema)?;
        Ok(Self {
            source: input.source.clone(),
            before: input.descriptor.clone(),
            after,
        })
    }

    pub fn source(&self) -> &AiBlackboardSourceVersion {
        &self.source
    }
    pub fn before(&self) -> &AiBlackboardSchemaDescriptor {
        &self.before
    }
    pub fn after(&self) -> &AiBlackboardSchemaDescriptor {
        &self.after
    }

    /// Whole-value equality cannot prevent ABA. The original host must separately
    /// check document/source/revision/lease, budget and one CAS/history transaction.
    pub fn forward_from(
        &self,
        current: &AiBlackboardSchemaDescriptor,
    ) -> Option<&AiBlackboardSchemaDescriptor> {
        (current == &self.before).then_some(&self.after)
    }

    pub fn inverse_from(
        &self,
        current: &AiBlackboardSchemaDescriptor,
    ) -> Option<&AiBlackboardSchemaDescriptor> {
        (current == &self.after).then_some(&self.before)
    }
}
