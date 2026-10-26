use zircon_runtime::core::framework::ai::AiBlackboardKeyDescriptor;

use super::super::search::matches_query;
use super::menu_types::{
    AiBlackboardKeyMenuRow, AiBlackboardMenuCandidate, AiBlackboardMenuCommand as Command,
    AiBlackboardMenuDisabledReason as Disabled, AiBlackboardMenuError as Error,
    AiBlackboardMenuRequest as Request, AiBlackboardMenuRow,
    AiBlackboardReferenceKnowledge as Knowledge,
};
use super::schema_edit::AiBlackboardSchemaEditCandidate;
use super::AiBlackboardReferencePlan;

/// Borrows the actual immutable schema/reference plan and current selection.
/// Rebuild after any source, selection, association or provider lifecycle change.
pub struct AiBlackboardMenuAdapter<'a, 's> {
    plan: &'a AiBlackboardReferencePlan<'s>,
    selection: &'a [&'a str],
}

impl<'a, 's> AiBlackboardMenuAdapter<'a, 's> {
    pub fn new(plan: &'a AiBlackboardReferencePlan<'s>, selection: &'a [&'a str]) -> Self {
        Self { plan, selection }
    }

    pub fn rows<'q>(&'q self, query: &'q str) -> impl Iterator<Item = AiBlackboardMenuRow> + 'q {
        [
            (Command::CreateKey, "New Key", "create_key"),
            (Command::RenameKey, "Rename Key", "rename_key"),
            (Command::ChangeType, "Change Key Type", "change_type"),
            (Command::DeleteKey, "Delete Key", "delete_key"),
            (
                Command::EditRuntimeValue,
                "Edit Runtime Value",
                "edit_runtime_value",
            ),
        ]
        .into_iter()
        .filter(move |(_, text, value)| matches_query(query, text, value))
        .map(move |(command, text, value)| AiBlackboardMenuRow {
            command,
            text,
            value,
            disabled: self.eligibility(command).err(),
        })
    }

    pub fn key_rows<'q>(
        &'q self,
        query: &'q str,
    ) -> impl Iterator<Item = AiBlackboardKeyMenuRow<'q>> + 'q {
        self.plan
            .schema
            .descriptor
            .keys
            .iter()
            .filter(move |key| matches_query(query, &key.key, &key.value_type))
            .map(move |key| AiBlackboardKeyMenuRow {
                key,
                selected: self.selection.contains(&key.key.as_str()),
                references: self.reference_knowledge(&key.key),
            })
    }

    pub fn eligibility(&self, command: Command) -> Result<(), Disabled> {
        if command == Command::EditRuntimeValue {
            return Err(Disabled::RuntimeValuesReadOnly);
        }
        if !self.plan.schema.writable {
            return Err(Disabled::ReadOnly);
        }
        // A new definition does not rewrite existing keys or assert reference
        // completeness. Required/default producer compatibility still belongs to
        // the original host's publication gate, not to this schema-only candidate.
        if command == Command::CreateKey {
            return Ok(());
        }
        let key = self.selected()?;
        let Knowledge::InSuppliedScope(count) = self.reference_knowledge(&key.key) else {
            return Err(Disabled::UnknownReferences);
        };
        match command {
            Command::RenameKey => {
                if self.plan.references.iter().any(|reference| {
                    reference.original.value.as_string() == Some(key.key.as_str())
                        && !self.plan.trees[reference.tree_index].writable
                }) {
                    return Err(Disabled::ReferencedTreeReadOnly);
                }
                Ok(())
            }
            Command::ChangeType | Command::DeleteKey => {
                if count > 0 {
                    Err(Disabled::KeyReferenced)
                } else {
                    Err(Disabled::SchemaEvolutionUnsupported)
                }
            }
            _ => unreachable!("create and runtime-value commands returned above"),
        }
    }

    /// Called only after original host allocation admission. Returns business
    /// values only; the original host must not write history on no-op or error.
    pub fn prepare(
        &self,
        request: Request<'_>,
    ) -> Result<Option<AiBlackboardMenuCandidate>, Error> {
        self.eligibility(request.command())
            .map_err(Error::Disabled)?;
        match request {
            Request::CreateKey { index, key } => Ok(Some(AiBlackboardMenuCandidate::CreateKey(
                AiBlackboardSchemaEditCandidate::create_key(self.plan.schema, index, key)?,
            ))),
            Request::RenameKey { new_name } => {
                let old = &self.selected().map_err(Error::Disabled)?.key;
                Ok(self
                    .plan
                    .rename_key(old, new_name)?
                    .map(AiBlackboardMenuCandidate::RenameKey))
            }
            _ => unreachable!("unsupported schema/runtime edits rejected before preparation"),
        }
    }

    fn selected(&self) -> Result<&AiBlackboardKeyDescriptor, Disabled> {
        let key = match self.selection {
            [] => return Err(Disabled::NoSelection),
            [key] => *key,
            _ => return Err(Disabled::MultipleSelection),
        };
        self.plan
            .schema
            .descriptor
            .keys
            .iter()
            .find(|entry| entry.key == key)
            .ok_or(Disabled::MissingKey)
    }

    fn reference_knowledge(&self, key: &str) -> Knowledge<'_> {
        if !self.plan.blockers.is_empty() {
            return Knowledge::Unknown(&self.plan.blockers);
        }
        Knowledge::InSuppliedScope(
            self.plan
                .references
                .iter()
                .filter(|reference| reference.original.value.as_string() == Some(key))
                .count(),
        )
    }
}
