use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::Arc;

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
    creation_menu: Arc<AssetCreationMenuGeneration>,
}

#[derive(Clone, Debug, Default)]
pub struct AssetCreationMenuGeneration {
    generation: u64,
    entries: Arc<[AssetCreationMenuEntry]>,
    action_index: Arc<HashMap<String, usize>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssetCreationMenuEntry {
    action_id: String,
    raw_item: String,
    asset_type: AssetTypeId,
    template_id: String,
}

impl AssetCreationMenuGeneration {
    fn compile(generation: u64, entries: &BTreeMap<AssetTypeId, MaterializedEntry>) -> Self {
        let candidates = entries
            .values()
            .flat_map(|entry| {
                entry
                    .definition
                    .creation_templates()
                    .iter()
                    .map(|template| {
                        (
                            entry.definition.id().clone(),
                            template.id().to_owned(),
                            format!("Create {}", safe_menu_label(template.display_name())),
                        )
                    })
            })
            .collect::<Vec<_>>();
        let mut label_counts = BTreeMap::<String, usize>::new();
        for (_, _, label) in &candidates {
            *label_counts.entry(label.clone()).or_default() += 1;
        }

        let mut compiled = Vec::with_capacity(candidates.len());
        let mut action_index = HashMap::with_capacity(candidates.len());
        let mut used_labels = BTreeSet::new();
        let mut next_suffix_by_base = BTreeMap::new();
        for (ordinal, (asset_type, template_id, base_label)) in candidates.into_iter().enumerate() {
            let label = if label_counts.get(&base_label).copied().unwrap_or_default() > 1 {
                format!(
                    "{base_label} ({}/{})",
                    safe_menu_label(asset_type.as_str()),
                    safe_menu_label(&template_id)
                )
            } else {
                base_label
            };
            let label = unique_menu_label(label, &mut used_labels, &mut next_suffix_by_base);
            let action_id = format!("menu.item.asset_create.{generation}.{ordinal}");
            let index = compiled.len();
            compiled.push(AssetCreationMenuEntry {
                raw_item: format!("{label}|action={action_id},icon=plus"),
                action_id: action_id.clone(),
                asset_type,
                template_id,
            });
            action_index.insert(action_id, index);
        }

        Self {
            generation,
            entries: compiled.into(),
            action_index: Arc::new(action_index),
        }
    }

    pub const fn generation(&self) -> u64 {
        self.generation
    }

    pub fn entries(&self) -> &[AssetCreationMenuEntry] {
        &self.entries
    }

    pub fn action(&self, action_id: &str) -> Option<&AssetCreationMenuEntry> {
        self.action_index
            .get(action_id)
            .and_then(|index| self.entries.get(*index))
    }
}

impl AssetCreationMenuEntry {
    pub fn action_id(&self) -> &str {
        &self.action_id
    }

    pub fn raw_item(&self) -> &str {
        &self.raw_item
    }

    pub fn asset_type(&self) -> &AssetTypeId {
        &self.asset_type
    }

    pub fn template_id(&self) -> &str {
        &self.template_id
    }
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

    pub fn creation_menu_generation(&self) -> Arc<AssetCreationMenuGeneration> {
        Arc::clone(&self.creation_menu)
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
        let report = batch::apply_contributions(self, contributions);
        if report.creation_template_entry_count() > 0 {
            self.creation_menu = Arc::new(AssetCreationMenuGeneration::compile(
                self.generation,
                &self.entries,
            ));
        }
        report
    }
}

fn unique_menu_label(
    label: String,
    used_labels: &mut BTreeSet<String>,
    next_suffix_by_base: &mut BTreeMap<String, usize>,
) -> String {
    if used_labels.insert(label.clone()) {
        return label;
    }

    // A base cursor means each occupied suffix is skipped only once, even when many
    // distinct template identifiers normalize to the same visible label.
    let next_suffix = next_suffix_by_base.entry(label.clone()).or_insert(2);
    loop {
        let ordinal = *next_suffix;
        *next_suffix = ordinal
            .checked_add(1)
            .expect("asset creation menu label suffix exhausted");
        let candidate = format!("{label} {ordinal}");
        if used_labels.insert(candidate.clone()) {
            return candidate;
        }
    }
}

fn safe_menu_label(value: &str) -> String {
    let mut label = String::with_capacity(value.len());
    let mut pending_space = false;
    for character in value.chars() {
        if character == '|' || character.is_control() || character.is_whitespace() {
            pending_space |= !label.is_empty();
            continue;
        }
        if pending_space {
            label.push(' ');
            pending_space = false;
        }
        label.push(character);
    }
    if label.is_empty() {
        "Asset".to_string()
    } else {
        label
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
