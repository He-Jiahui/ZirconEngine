use std::collections::BTreeMap;

use super::{
    AssetToolkitDescriptor, AssetTypeContribution, AssetTypeDefinition, AssetTypeId,
    AssetTypePresentation, AssetTypeRegistryError,
};

#[derive(Clone, Debug, Default)]
pub struct AssetTypeRegistry {
    entries: BTreeMap<AssetTypeId, MaterializedEntry>,
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

    pub fn get(&self, asset_type: &AssetTypeId) -> Option<&AssetTypeDefinition> {
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
        let asset_type = contribution.asset_type.clone();
        let candidate = match self.entries.get(&asset_type) {
            Some(existing) => merge_existing(existing.clone(), &owner, contribution)?,
            None => materialize_new(&owner, contribution)?,
        };
        self.entries.insert(asset_type, candidate);
        Ok(())
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
    validate_context_commands(&asset_type, &contribution.context_commands)?;
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
            creation_templates: sorted_creation_templates(contribution.creation_templates),
            context_commands: sorted_context_commands(contribution.context_commands),
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

fn merge_existing(
    mut entry: MaterializedEntry,
    owner: &str,
    contribution: AssetTypeContribution,
) -> Result<MaterializedEntry, AssetTypeRegistryError> {
    if let Some(value) = contribution.runtime_kind {
        claim_field(
            &contribution.asset_type,
            "runtime_kind",
            &entry.owners.runtime_kind,
            owner,
        )?;
        entry.definition.runtime_kind = Some(value);
        entry.owners.runtime_kind = Some(owner.to_owned());
    }
    if let Some(value) = contribution.source_write_policy {
        claim_field(
            &contribution.asset_type,
            "source_write_policy",
            &entry.owners.source_write_policy,
            owner,
        )?;
        entry.definition.source_write_policy = value;
        entry.owners.source_write_policy = Some(owner.to_owned());
    }
    if let Some(value) = contribution.presentation {
        claim_field(
            &contribution.asset_type,
            "presentation",
            &entry.owners.presentation,
            owner,
        )?;
        validate_presentation(&contribution.asset_type, &value)?;
        entry.definition.presentation = value;
        entry.owners.presentation = Some(owner.to_owned());
    }
    if let Some(value) = contribution.thumbnail {
        claim_field(
            &contribution.asset_type,
            "thumbnail",
            &entry.owners.thumbnail,
            owner,
        )?;
        entry.definition.thumbnail = value;
        entry.owners.thumbnail = Some(owner.to_owned());
    }
    if let Some(value) = contribution.toolkit {
        claim_field(
            &contribution.asset_type,
            "toolkit",
            &entry.owners.toolkit,
            owner,
        )?;
        validate_toolkit(&contribution.asset_type, Some(&value))?;
        entry.definition.toolkit = Some(value);
        entry.owners.toolkit = Some(owner.to_owned());
    }
    validate_creation_templates(&contribution.asset_type, &contribution.creation_templates)?;
    for template in contribution.creation_templates {
        let entry_id = template.id().to_owned();
        if let Some(first_owner) = entry.owners.creation_templates.get(&entry_id) {
            return Err(AssetTypeRegistryError::DuplicateEntryOwner {
                asset_type: contribution.asset_type,
                collection: "creation_templates",
                entry_id,
                first_owner: first_owner.clone(),
                second_owner: owner.to_owned(),
            });
        }
        entry
            .owners
            .creation_templates
            .insert(entry_id, owner.to_owned());
        entry.definition.creation_templates.push(template);
    }
    entry
        .definition
        .creation_templates
        .sort_by(|left, right| left.id().cmp(right.id()));
    validate_context_commands(&contribution.asset_type, &contribution.context_commands)?;
    for command in contribution.context_commands {
        let entry_id = command.id().to_owned();
        if let Some(first_owner) = entry.owners.context_commands.get(&entry_id) {
            return Err(AssetTypeRegistryError::DuplicateEntryOwner {
                asset_type: contribution.asset_type,
                collection: "context_commands",
                entry_id,
                first_owner: first_owner.clone(),
                second_owner: owner.to_owned(),
            });
        }
        entry
            .owners
            .context_commands
            .insert(entry_id, owner.to_owned());
        entry.definition.context_commands.push(command);
    }
    entry
        .definition
        .context_commands
        .sort_by(|left, right| left.id().cmp(right.id()));
    Ok(entry)
}

fn claim_field(
    asset_type: &AssetTypeId,
    field: &'static str,
    first_owner: &Option<String>,
    second_owner: &str,
) -> Result<(), AssetTypeRegistryError> {
    if let Some(first_owner) = first_owner {
        return Err(AssetTypeRegistryError::DuplicateFieldOwner {
            asset_type: asset_type.clone(),
            field,
            first_owner: first_owner.clone(),
            second_owner: second_owner.to_owned(),
        });
    }
    Ok(())
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

fn sorted_creation_templates(
    mut templates: Vec<super::AssetCreationTemplateDescriptor>,
) -> Vec<super::AssetCreationTemplateDescriptor> {
    templates.sort_by(|left, right| left.id().cmp(right.id()));
    templates
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

fn sorted_context_commands(
    mut commands: Vec<super::AssetContextCommandDescriptor>,
) -> Vec<super::AssetContextCommandDescriptor> {
    commands.sort_by(|left, right| left.id().cmp(right.id()));
    commands
}
