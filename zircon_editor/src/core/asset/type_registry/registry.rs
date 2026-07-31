use std::collections::{BTreeMap, BTreeSet};

use super::{
    AssetToolkitDescriptor, AssetTypeContribution, AssetTypeDefinition, AssetTypeId,
    AssetTypePresentation, AssetTypeRegistryError,
};

mod batch;

pub(crate) use batch::AssetTypeRegistryBatchReport;

#[derive(Clone, Debug, Default)]
pub struct AssetTypeRegistry {
    entries: BTreeMap<AssetTypeId, MaterializedEntry>,
    generation: u64,
}

#[derive(Clone, Debug)]
struct MaterializedEntry {
    definition: AssetTypeDefinition,
    owners: FieldOwners,
}

#[derive(Clone, Debug, Default)]
struct FieldOwners {
    runtime_kind: Option<String>,
    source_write_policy: Option<String>,
    presentation: Option<String>,
    thumbnail: Option<String>,
    toolkit: Option<String>,
    creation_templates: BTreeMap<String, String>,
    context_commands: BTreeMap<String, String>,
}

impl AssetTypeRegistry {
    pub fn with_builtins() -> Result<Self, AssetTypeRegistryError> {
        super::builtin::builtin_registry()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub fn get(&self, asset_type: &AssetTypeId) -> Option<&AssetTypeDefinition> {
        self.entries.get(asset_type).map(|entry| &entry.definition)
    }

    pub(super) fn get_by_id(&self, asset_type: &str) -> Option<&AssetTypeDefinition> {
        self.entries.get(asset_type).map(|entry| &entry.definition)
    }

    pub fn definitions(&self) -> impl Iterator<Item = &AssetTypeDefinition> {
        self.entries.values().map(|entry| &entry.definition)
    }

    pub fn apply_contribution(
        &mut self,
        owner: impl Into<String>,
        contribution: AssetTypeContribution,
    ) -> Result<(), AssetTypeRegistryError> {
        let owner = owner.into();
        let report = self.apply_contributions([(owner, contribution)]);
        match report.into_errors().pop() {
            Some((_, error)) => Err(error),
            None => Ok(()),
        }
    }

    pub(crate) fn apply_contributions<I, O>(
        &mut self,
        contributions: I,
    ) -> AssetTypeRegistryBatchReport
    where
        I: IntoIterator<Item = (O, AssetTypeContribution)>,
        O: Into<String>,
    {
        batch::apply_contributions(self, contributions)
    }
}

fn materialize_new(
    owner: &str,
    contribution: AssetTypeContribution,
) -> Result<MaterializedEntry, AssetTypeRegistryError> {
    let asset_type = contribution.asset_type;
    let mut missing_fields = Vec::new();
    if contribution.presentation.is_none() {
        missing_fields.push("presentation");
    }
    if contribution.source_write_policy.is_none() {
        missing_fields.push("source_write_policy");
    }
    if contribution.thumbnail.is_none() {
        missing_fields.push("thumbnail");
    }
    if !missing_fields.is_empty() {
        return Err(AssetTypeRegistryError::IncompleteDefinition {
            asset_type,
            missing_fields,
        });
    }

    let presentation =
        contribution
            .presentation
            .ok_or_else(|| AssetTypeRegistryError::IncompleteDefinition {
                asset_type: asset_type.clone(),
                missing_fields: vec!["presentation"],
            })?;
    let source_write_policy = contribution.source_write_policy.ok_or_else(|| {
        AssetTypeRegistryError::IncompleteDefinition {
            asset_type: asset_type.clone(),
            missing_fields: vec!["source_write_policy"],
        }
    })?;
    validate_presentation(&asset_type, &presentation)?;
    let thumbnail =
        contribution
            .thumbnail
            .ok_or_else(|| AssetTypeRegistryError::IncompleteDefinition {
                asset_type: asset_type.clone(),
                missing_fields: vec!["thumbnail"],
            })?;
    validate_toolkit(&asset_type, contribution.toolkit.as_ref())?;
    let has_toolkit = contribution.toolkit.is_some();
    validate_creation_templates(&asset_type, &contribution.creation_templates)?;
    validate_new_creation_template_owners(&asset_type, owner, &contribution.creation_templates)?;
    validate_context_commands(&asset_type, &contribution.context_commands)?;
    validate_new_context_command_owners(&asset_type, owner, &contribution.context_commands)?;
    let creation_template_owners = contribution
        .creation_templates
        .iter()
        .map(|template| (template.id().to_owned(), owner.to_owned()))
        .collect();
    let context_command_owners = contribution
        .context_commands
        .iter()
        .map(|command| (command.id().to_owned(), owner.to_owned()))
        .collect();

    Ok(MaterializedEntry {
        definition: AssetTypeDefinition {
            id: asset_type,
            runtime_kind: contribution.runtime_kind,
            source_write_policy,
            presentation,
            thumbnail,
            toolkit: contribution.toolkit,
            creation_templates: contribution.creation_templates,
            context_commands: contribution.context_commands,
        },
        owners: FieldOwners {
            runtime_kind: contribution.runtime_kind.map(|_| owner.to_owned()),
            source_write_policy: Some(owner.to_owned()),
            presentation: Some(owner.to_owned()),
            thumbnail: Some(owner.to_owned()),
            toolkit: has_toolkit.then(|| owner.to_owned()),
            creation_templates: creation_template_owners,
            context_commands: context_command_owners,
        },
    })
}

fn validate_presentation(
    asset_type: &AssetTypeId,
    presentation: &AssetTypePresentation,
) -> Result<(), AssetTypeRegistryError> {
    if let Some(field) = presentation.first_empty_field() {
        return Err(AssetTypeRegistryError::EmptyRequiredField {
            asset_type: asset_type.clone(),
            field,
        });
    }
    Ok(())
}

fn validate_toolkit(
    asset_type: &AssetTypeId,
    toolkit: Option<&AssetToolkitDescriptor>,
) -> Result<(), AssetTypeRegistryError> {
    if toolkit.is_some_and(|value| value.view_id().is_empty()) {
        return Err(AssetTypeRegistryError::EmptyRequiredField {
            asset_type: asset_type.clone(),
            field: "toolkit.view_id",
        });
    }
    Ok(())
}

fn validate_creation_templates(
    asset_type: &AssetTypeId,
    templates: &[super::AssetCreationTemplateDescriptor],
) -> Result<(), AssetTypeRegistryError> {
    for template in templates {
        for (field, value) in [
            ("creation_template.id", template.id()),
            ("creation_template.display_name", template.display_name()),
        ] {
            if value.is_empty() {
                return Err(AssetTypeRegistryError::EmptyRequiredField {
                    asset_type: asset_type.clone(),
                    field,
                });
            }
        }
    }
    Ok(())
}

fn validate_new_creation_template_owners(
    asset_type: &AssetTypeId,
    owner: &str,
    templates: &[super::AssetCreationTemplateDescriptor],
) -> Result<(), AssetTypeRegistryError> {
    let mut entry_ids = BTreeSet::new();
    for template in templates {
        if !entry_ids.insert(template.id()) {
            return Err(AssetTypeRegistryError::DuplicateEntryOwner {
                asset_type: asset_type.clone(),
                collection: "creation_templates",
                entry_id: template.id().to_owned(),
                first_owner: owner.to_owned(),
                second_owner: owner.to_owned(),
            });
        }
    }
    Ok(())
}

fn validate_context_commands(
    asset_type: &AssetTypeId,
    commands: &[super::AssetContextCommandDescriptor],
) -> Result<(), AssetTypeRegistryError> {
    for command in commands {
        for (field, value) in [
            ("context_command.id", command.id()),
            ("context_command.display_name", command.display_name()),
        ] {
            if value.is_empty() {
                return Err(AssetTypeRegistryError::EmptyRequiredField {
                    asset_type: asset_type.clone(),
                    field,
                });
            }
        }
    }
    Ok(())
}

fn validate_new_context_command_owners(
    asset_type: &AssetTypeId,
    owner: &str,
    commands: &[super::AssetContextCommandDescriptor],
) -> Result<(), AssetTypeRegistryError> {
    let mut entry_ids = BTreeSet::new();
    for command in commands {
        if !entry_ids.insert(command.id()) {
            return Err(AssetTypeRegistryError::DuplicateEntryOwner {
                asset_type: asset_type.clone(),
                collection: "context_commands",
                entry_id: command.id().to_owned(),
                first_owner: owner.to_owned(),
                second_owner: owner.to_owned(),
            });
        }
    }
    Ok(())
}
