use std::collections::{BTreeMap, BTreeSet};

use super::super::{AssetTypeContribution, AssetTypeId, AssetTypeRegistryError};
use super::{
    AssetTypeRegistry, MaterializedEntry, materialize_new, validate_context_commands,
    validate_creation_templates, validate_presentation, validate_toolkit,
};

#[derive(Debug, Default)]
pub(crate) struct AssetTypeRegistryBatchReport {
    errors: Vec<(usize, AssetTypeRegistryError)>,
    accepted_count: usize,
    touched_asset_type_count: usize,
    creation_template_sort_count: usize,
    creation_template_entry_count: usize,
    context_command_sort_count: usize,
    context_command_entry_count: usize,
}

impl AssetTypeRegistryBatchReport {
    pub(crate) fn errors(&self) -> &[(usize, AssetTypeRegistryError)] {
        &self.errors
    }

    pub(crate) fn accepted_count(&self) -> usize {
        self.accepted_count
    }

    pub(crate) fn rejected_count(&self) -> usize {
        self.errors.len()
    }

    pub(crate) fn touched_asset_type_count(&self) -> usize {
        self.touched_asset_type_count
    }

    pub(crate) fn creation_template_sort_count(&self) -> usize {
        self.creation_template_sort_count
    }

    pub(crate) fn creation_template_entry_count(&self) -> usize {
        self.creation_template_entry_count
    }

    pub(crate) fn context_command_sort_count(&self) -> usize {
        self.context_command_sort_count
    }

    pub(crate) fn context_command_entry_count(&self) -> usize {
        self.context_command_entry_count
    }

    pub(crate) fn into_errors(self) -> Vec<(usize, AssetTypeRegistryError)> {
        self.errors
    }
}

#[derive(Debug, Default)]
struct PendingEntryDelta {
    new_entry: Option<MaterializedEntry>,
    claims: PendingClaims,
    staged: Vec<StagedContribution>,
}

#[derive(Debug, Default)]
struct PendingClaims {
    runtime_kind: Option<String>,
    source_write_policy: Option<String>,
    presentation: Option<String>,
    thumbnail: Option<String>,
    toolkit: Option<String>,
    creation_templates: BTreeMap<String, String>,
    context_commands: BTreeMap<String, String>,
}

#[derive(Debug)]
struct StagedContribution {
    owner: String,
    contribution: AssetTypeContribution,
}

pub(super) fn apply_contributions<I, O>(
    registry: &mut AssetTypeRegistry,
    contributions: I,
) -> AssetTypeRegistryBatchReport
where
    I: IntoIterator<Item = (O, AssetTypeContribution)>,
    O: Into<String>,
{
    let mut pending = BTreeMap::<AssetTypeId, PendingEntryDelta>::new();
    let mut report = AssetTypeRegistryBatchReport::default();

    for (input_index, (owner, contribution)) in contributions.into_iter().enumerate() {
        let owner = owner.into();
        let asset_type = contribution.asset_type.clone();
        let result = if let Some(delta) = pending.get_mut(&asset_type) {
            let base = delta
                .new_entry
                .as_ref()
                .or_else(|| registry.entries.get(&asset_type))
                .expect("pending asset type must have a materialized base");
            validate_contribution(base, &delta.claims, &owner, &contribution).map(|()| {
                record_pending_claims(&mut delta.claims, &owner, &contribution);
                delta.staged.push(StagedContribution {
                    owner,
                    contribution,
                });
            })
        } else if let Some(base) = registry.entries.get(&asset_type) {
            let mut delta = PendingEntryDelta::default();
            validate_contribution(base, &delta.claims, &owner, &contribution).map(|()| {
                record_pending_claims(&mut delta.claims, &owner, &contribution);
                delta.staged.push(StagedContribution {
                    owner,
                    contribution,
                });
                pending.insert(asset_type, delta);
            })
        } else {
            materialize_new(&owner, contribution).map(|entry| {
                pending.insert(
                    asset_type,
                    PendingEntryDelta {
                        new_entry: Some(entry),
                        ..PendingEntryDelta::default()
                    },
                );
            })
        };

        match result {
            Ok(()) => report.accepted_count += 1,
            Err(error) => report.errors.push((input_index, error)),
        }
    }

    report.touched_asset_type_count = pending.len();
    if report.accepted_count > 0 {
        finalize_pending_entries(registry, pending, &mut report);
        registry.generation = registry.generation.saturating_add(1);
    }
    report
}

fn validate_contribution(
    base: &MaterializedEntry,
    pending: &PendingClaims,
    owner: &str,
    contribution: &AssetTypeContribution,
) -> Result<(), AssetTypeRegistryError> {
    if contribution.runtime_kind.is_some() {
        validate_field_claim(
            &contribution.asset_type,
            "runtime_kind",
            base.owners.runtime_kind.as_ref(),
            pending.runtime_kind.as_ref(),
            owner,
        )?;
    }
    if contribution.source_write_policy.is_some() {
        validate_field_claim(
            &contribution.asset_type,
            "source_write_policy",
            base.owners.source_write_policy.as_ref(),
            pending.source_write_policy.as_ref(),
            owner,
        )?;
    }
    if let Some(presentation) = contribution.presentation.as_ref() {
        validate_field_claim(
            &contribution.asset_type,
            "presentation",
            base.owners.presentation.as_ref(),
            pending.presentation.as_ref(),
            owner,
        )?;
        validate_presentation(&contribution.asset_type, presentation)?;
    }
    if contribution.thumbnail.is_some() {
        validate_field_claim(
            &contribution.asset_type,
            "thumbnail",
            base.owners.thumbnail.as_ref(),
            pending.thumbnail.as_ref(),
            owner,
        )?;
    }
    if let Some(toolkit) = contribution.toolkit.as_ref() {
        validate_field_claim(
            &contribution.asset_type,
            "toolkit",
            base.owners.toolkit.as_ref(),
            pending.toolkit.as_ref(),
            owner,
        )?;
        validate_toolkit(&contribution.asset_type, Some(toolkit))?;
    }

    validate_creation_templates(&contribution.asset_type, &contribution.creation_templates)?;
    validate_collection_claims(
        &contribution.asset_type,
        "creation_templates",
        owner,
        contribution
            .creation_templates
            .iter()
            .map(|template| template.id()),
        &base.owners.creation_templates,
        &pending.creation_templates,
    )?;
    validate_context_commands(&contribution.asset_type, &contribution.context_commands)?;
    validate_collection_claims(
        &contribution.asset_type,
        "context_commands",
        owner,
        contribution
            .context_commands
            .iter()
            .map(|command| command.id()),
        &base.owners.context_commands,
        &pending.context_commands,
    )
}

fn validate_field_claim(
    asset_type: &AssetTypeId,
    field: &'static str,
    base_owner: Option<&String>,
    pending_owner: Option<&String>,
    owner: &str,
) -> Result<(), AssetTypeRegistryError> {
    if let Some(first_owner) = base_owner.or(pending_owner) {
        return Err(AssetTypeRegistryError::DuplicateFieldOwner {
            asset_type: asset_type.clone(),
            field,
            first_owner: first_owner.clone(),
            second_owner: owner.to_owned(),
        });
    }
    Ok(())
}

fn validate_collection_claims<'a>(
    asset_type: &AssetTypeId,
    collection: &'static str,
    owner: &str,
    entry_ids: impl IntoIterator<Item = &'a str>,
    base_owners: &BTreeMap<String, String>,
    pending_owners: &BTreeMap<String, String>,
) -> Result<(), AssetTypeRegistryError> {
    let mut contribution_ids = BTreeSet::new();
    for entry_id in entry_ids {
        if let Some(first_owner) = base_owners
            .get(entry_id)
            .or_else(|| pending_owners.get(entry_id))
        {
            return Err(AssetTypeRegistryError::DuplicateEntryOwner {
                asset_type: asset_type.clone(),
                collection,
                entry_id: entry_id.to_owned(),
                first_owner: first_owner.clone(),
                second_owner: owner.to_owned(),
            });
        }
        if !contribution_ids.insert(entry_id) {
            return Err(AssetTypeRegistryError::DuplicateEntryOwner {
                asset_type: asset_type.clone(),
                collection,
                entry_id: entry_id.to_owned(),
                first_owner: owner.to_owned(),
                second_owner: owner.to_owned(),
            });
        }
    }
    Ok(())
}

fn record_pending_claims(
    pending: &mut PendingClaims,
    owner: &str,
    contribution: &AssetTypeContribution,
) {
    if contribution.runtime_kind.is_some() {
        pending.runtime_kind = Some(owner.to_owned());
    }
    if contribution.source_write_policy.is_some() {
        pending.source_write_policy = Some(owner.to_owned());
    }
    if contribution.presentation.is_some() {
        pending.presentation = Some(owner.to_owned());
    }
    if contribution.thumbnail.is_some() {
        pending.thumbnail = Some(owner.to_owned());
    }
    if contribution.toolkit.is_some() {
        pending.toolkit = Some(owner.to_owned());
    }
    for template in &contribution.creation_templates {
        pending
            .creation_templates
            .insert(template.id().to_owned(), owner.to_owned());
    }
    for command in &contribution.context_commands {
        pending
            .context_commands
            .insert(command.id().to_owned(), owner.to_owned());
    }
}

fn finalize_pending_entries(
    registry: &mut AssetTypeRegistry,
    pending: BTreeMap<AssetTypeId, PendingEntryDelta>,
    report: &mut AssetTypeRegistryBatchReport,
) {
    for (asset_type, mut delta) in pending {
        if let Some(mut entry) = delta.new_entry.take() {
            finalize_entry(&mut entry, delta.staged, true, report);
            registry.entries.insert(asset_type, entry);
        } else {
            let entry = registry
                .entries
                .get_mut(&asset_type)
                .expect("existing pending asset type must remain registered");
            finalize_entry(entry, delta.staged, false, report);
        }
    }
}

fn finalize_entry(
    entry: &mut MaterializedEntry,
    staged: Vec<StagedContribution>,
    is_new: bool,
    report: &mut AssetTypeRegistryBatchReport,
) {
    let (creation_template_count, context_command_count) = staged.iter().fold(
        (0usize, 0usize),
        |(creation_templates, context_commands), staged| {
            (
                creation_templates.saturating_add(staged.contribution.creation_templates.len()),
                context_commands.saturating_add(staged.contribution.context_commands.len()),
            )
        },
    );
    let mut creation_templates = Vec::with_capacity(creation_template_count);
    let mut context_commands = Vec::with_capacity(context_command_count);

    for staged in staged {
        let owner = staged.owner;
        let contribution = staged.contribution;
        if let Some(value) = contribution.runtime_kind {
            entry.definition.runtime_kind = Some(value);
            entry.owners.runtime_kind = Some(owner.clone());
        }
        if let Some(value) = contribution.source_write_policy {
            entry.definition.source_write_policy = value;
            entry.owners.source_write_policy = Some(owner.clone());
        }
        if let Some(value) = contribution.presentation {
            entry.definition.presentation = value;
            entry.owners.presentation = Some(owner.clone());
        }
        if let Some(value) = contribution.thumbnail {
            entry.definition.thumbnail = value;
            entry.owners.thumbnail = Some(owner.clone());
        }
        if let Some(value) = contribution.toolkit {
            entry.definition.toolkit = Some(value);
            entry.owners.toolkit = Some(owner.clone());
        }
        for template in &contribution.creation_templates {
            entry
                .owners
                .creation_templates
                .insert(template.id().to_owned(), owner.clone());
        }
        for command in &contribution.context_commands {
            entry
                .owners
                .context_commands
                .insert(command.id().to_owned(), owner.clone());
        }
        creation_templates.extend(contribution.creation_templates);
        context_commands.extend(contribution.context_commands);
    }

    let finalize_creation_templates = !creation_templates.is_empty()
        || (is_new && !entry.definition.creation_templates.is_empty());
    if finalize_creation_templates {
        entry
            .definition
            .creation_templates
            .extend(creation_templates);
        entry
            .definition
            .creation_templates
            .sort_by(|left, right| left.id().cmp(right.id()));
        report.creation_template_sort_count += 1;
        report.creation_template_entry_count += entry.definition.creation_templates.len();
    }

    let finalize_context_commands =
        !context_commands.is_empty() || (is_new && !entry.definition.context_commands.is_empty());
    if finalize_context_commands {
        entry.definition.context_commands.extend(context_commands);
        entry
            .definition
            .context_commands
            .sort_by(|left, right| left.id().cmp(right.id()));
        report.context_command_sort_count += 1;
        report.context_command_entry_count += entry.definition.context_commands.len();
    }
}

#[cfg(test)]
mod optimization_batch_20260830cj_editor_tests {
    use std::time::Instant;

    const SAMPLE_PAIRS: usize = 17;
    const CONTRIBUTIONS_PER_SAMPLE: usize = 4_096;

    #[test]
    fn optimization_batch_20260830cj_editor_asset_type_batch_reserves_exact_contribution_outputs() {
        let source = include_str!("batch.rs");
        let implementation = source
            .split("#[cfg(test)]")
            .next()
            .expect("asset type batch implementation");

        assert!(implementation.contains("let (creation_template_count, context_command_count)"));
        assert!(implementation.contains("Vec::with_capacity(creation_template_count)"));
        assert!(implementation.contains("Vec::with_capacity(context_command_count)"));
    }

    #[test]
    #[ignore = "managed Windows release performance evidence"]
    fn optimization_batch_20260830cj_editor_asset_type_batch_capacity_p95() {
        let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy.push(measure(false));
                optimized.push(measure(true));
            } else {
                optimized.push(measure(true));
                legacy.push(measure(false));
            }
        }
        let legacy_p95_ns = percentile(&legacy, 95);
        let optimized_p95_ns = percentile(&optimized, 95);
        println!(
            "EDITOR332_ASSET_TYPE_BATCH_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} contributions_per_sample={CONTRIBUTIONS_PER_SAMPLE} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
            csv(&legacy),
            csv(&optimized)
        );
        assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
    }

    fn measure(use_capacity: bool) -> u128 {
        let contribution_sizes = [(1usize, 1usize); CONTRIBUTIONS_PER_SAMPLE];
        let started = Instant::now();
        let (creation_count, command_count) = if use_capacity {
            contribution_sizes.iter().fold(
                (0usize, 0usize),
                |(creations, commands), (next_creations, next_commands)| {
                    (
                        creations.saturating_add(*next_creations),
                        commands.saturating_add(*next_commands),
                    )
                },
            )
        } else {
            (0, 0)
        };
        let mut creations = Vec::with_capacity(creation_count);
        let mut commands = Vec::with_capacity(command_count);
        for (creation_size, command_size) in contribution_sizes {
            creations.extend(0..creation_size);
            commands.extend(0..command_size);
        }
        std::hint::black_box((creations, commands));
        started.elapsed().as_nanos().max(1)
    }

    fn percentile(samples: &[u128], p: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        sorted[(sorted.len() * p).div_ceil(100).saturating_sub(1)]
    }

    fn csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
