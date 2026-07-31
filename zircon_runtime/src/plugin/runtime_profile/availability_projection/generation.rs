use std::collections::HashMap;

use crate::builtin::RuntimePluginId;
use crate::core::framework::platform::RuntimeTargetMode;
use crate::plugin::{
    PluginMaturity, RuntimePluginAvailabilityCategory, RuntimePluginAvailabilityEntry,
    RuntimePluginAvailabilityReport, RuntimePluginDescriptor, RuntimePluginRegistrationReport,
};

const PRIMARY_AVAILABILITY_CATEGORY_COUNT: usize = 7;

const fn primary_category_index(category: RuntimePluginAvailabilityCategory) -> usize {
    match category {
        RuntimePluginAvailabilityCategory::Available => 0,
        RuntimePluginAvailabilityCategory::Linked => 1,
        RuntimePluginAvailabilityCategory::NativeDynamic => 2,
        RuntimePluginAvailabilityCategory::ExternalizedMissing => 3,
        RuntimePluginAvailabilityCategory::Stub => 4,
        RuntimePluginAvailabilityCategory::BlockedByTarget => 5,
        RuntimePluginAvailabilityCategory::BlockedByMaturity => 6,
        RuntimePluginAvailabilityCategory::MissingRequired => {
            panic!("missing-required is an index view, not a primary category")
        }
    }
}

/// Immutable category, lookup, and summary state for one availability generation.
///
/// A required row is stored once in `rows`; `missing_required` is an index view over that same
/// row. The legacy report is materialized only for export or diagnostics that require owned
/// serializable entries.
pub struct RuntimePluginAvailabilityGeneration<'a> {
    rows: Box<[RuntimePluginAvailabilityRow<'a>]>,
    categories: [Box<[usize]>; PRIMARY_AVAILABILITY_CATEGORY_COUNT],
    missing_required: Box<[usize]>,
    by_runtime_id: HashMap<RuntimePluginId, usize>,
    summary: RuntimePluginAvailabilitySummary,
}

/// A compact row borrowed by category or paged availability consumers.
///
/// The row intentionally stores no owned diagnostic reason text. Call
/// [`Self::detail`] only at a detail, export, or diagnostic boundary.
pub struct RuntimePluginAvailabilityRow<'a> {
    package_id: Option<&'a str>,
    runtime_id: RuntimePluginId,
    required: bool,
    maturity: PluginMaturity,
    category: RuntimePluginAvailabilityCategory,
    reason: RuntimePluginAvailabilityReason,
}

#[derive(Clone, Copy)]
pub(super) enum RuntimePluginAvailabilityReason {
    BuiltinAvailable,
    BuiltinUnavailable,
    MissingCatalog,
    TargetUnsupported(RuntimeTargetMode),
    Externalized,
    Stub,
    BelowMinimum(PluginMaturity),
    Linked,
    NativeDynamic,
    MissingProvider,
    Available,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RuntimePluginAvailabilitySummary {
    row_count: usize,
    category_counts: [usize; PRIMARY_AVAILABILITY_CATEGORY_COUNT],
    missing_required_count: usize,
}

impl RuntimePluginAvailabilityGeneration<'_> {
    pub fn entries(
        &self,
        category: RuntimePluginAvailabilityCategory,
    ) -> impl Iterator<Item = &RuntimePluginAvailabilityRow<'_>> {
        self.category_indexes(category)
            .iter()
            .map(|index| &self.rows[*index])
    }

    pub fn row_for(
        &self,
        runtime_id: RuntimePluginId,
    ) -> Option<&RuntimePluginAvailabilityRow<'_>> {
        self.by_runtime_id
            .get(&runtime_id)
            .map(|index| &self.rows[*index])
    }

    pub const fn summary(&self) -> RuntimePluginAvailabilitySummary {
        self.summary
    }

    pub fn materialize_report(&self) -> RuntimePluginAvailabilityReport {
        RuntimePluginAvailabilityReport {
            available: self
                .entries(RuntimePluginAvailabilityCategory::Available)
                .map(RuntimePluginAvailabilityRow::detail)
                .collect(),
            linked: self
                .entries(RuntimePluginAvailabilityCategory::Linked)
                .map(RuntimePluginAvailabilityRow::detail)
                .collect(),
            native_dynamic: self
                .entries(RuntimePluginAvailabilityCategory::NativeDynamic)
                .map(RuntimePluginAvailabilityRow::detail)
                .collect(),
            externalized_missing: self
                .entries(RuntimePluginAvailabilityCategory::ExternalizedMissing)
                .map(RuntimePluginAvailabilityRow::detail)
                .collect(),
            stub: self
                .entries(RuntimePluginAvailabilityCategory::Stub)
                .map(RuntimePluginAvailabilityRow::detail)
                .collect(),
            blocked_by_target: self
                .entries(RuntimePluginAvailabilityCategory::BlockedByTarget)
                .map(RuntimePluginAvailabilityRow::detail)
                .collect(),
            blocked_by_maturity: self
                .entries(RuntimePluginAvailabilityCategory::BlockedByMaturity)
                .map(RuntimePluginAvailabilityRow::detail)
                .collect(),
            missing_required: self
                .entries(RuntimePluginAvailabilityCategory::MissingRequired)
                .map(RuntimePluginAvailabilityRow::detail)
                .collect(),
        }
    }

    fn category_indexes(&self, category: RuntimePluginAvailabilityCategory) -> &[usize] {
        if category == RuntimePluginAvailabilityCategory::MissingRequired {
            return &self.missing_required;
        }
        &self.categories[primary_category_index(category)]
    }
}

impl RuntimePluginAvailabilityRow<'_> {
    pub fn id(&self) -> &str {
        self.package_id.unwrap_or_else(|| self.runtime_id.key())
    }

    pub fn runtime_id(&self) -> &RuntimePluginId {
        &self.runtime_id
    }

    pub const fn required(&self) -> bool {
        self.required
    }

    pub const fn maturity(&self) -> PluginMaturity {
        self.maturity
    }

    pub const fn category(&self) -> RuntimePluginAvailabilityCategory {
        self.category
    }

    pub fn detail(&self) -> RuntimePluginAvailabilityEntry {
        RuntimePluginAvailabilityEntry {
            id: self.id().to_string(),
            runtime_id: self.runtime_id.clone(),
            required: self.required,
            maturity: self.maturity,
            reason: self.reason.materialize(self.maturity),
        }
    }
}

impl RuntimePluginAvailabilityReason {
    fn materialize(self, maturity: PluginMaturity) -> String {
        match self {
            Self::BuiltinAvailable => {
                "plugin is provided by the built-in runtime domain".to_string()
            }
            Self::BuiltinUnavailable => {
                "built-in UI runtime is unavailable because the ui feature is disabled".to_string()
            }
            Self::MissingCatalog => "plugin is missing from runtime catalog".to_string(),
            Self::TargetUnsupported(target) => format!("target {target:?} is not supported"),
            Self::Externalized => {
                "plugin runtime is externalized and no linked registration was supplied".to_string()
            }
            Self::Stub => "plugin catalog entry is a stub".to_string(),
            Self::BelowMinimum(minimum) => {
                format!("plugin maturity {maturity:?} is below profile minimum {minimum:?}")
            }
            Self::Linked => "plugin runtime was supplied by linked registration".to_string(),
            Self::NativeDynamic => {
                "plugin runtime was supplied by native dynamic registration".to_string()
            }
            Self::MissingProvider => {
                "plugin runtime has no linked or native dynamic provider registration".to_string()
            }
            Self::Available => "plugin descriptor satisfies profile gates".to_string(),
        }
    }
}

pub(super) struct RuntimePluginAvailabilityDescriptorRef<'a> {
    pub(super) package_id: &'a str,
    pub(super) runtime_id: RuntimePluginId,
    pub(super) maturity: PluginMaturity,
    pub(super) target_modes: &'a [RuntimeTargetMode],
}

impl<'a> RuntimePluginAvailabilityDescriptorRef<'a> {
    pub(super) fn from_descriptor(descriptor: &'a RuntimePluginDescriptor) -> Self {
        Self {
            package_id: descriptor.package_id(),
            runtime_id: descriptor.runtime_id(),
            maturity: descriptor.maturity(),
            target_modes: descriptor.target_modes(),
        }
    }

    pub(super) fn from_registration(
        registration: &'a RuntimePluginRegistrationReport,
    ) -> Option<Self> {
        Some(Self {
            package_id: &registration.package_manifest.id,
            runtime_id: RuntimePluginId::parse_key(&registration.project_selection.id)?,
            maturity: registration.package_manifest.maturity,
            target_modes: &registration.package_manifest.supported_targets,
        })
    }
}

pub(super) fn row_from_descriptor<'a>(
    descriptor: &RuntimePluginAvailabilityDescriptorRef<'a>,
    required: bool,
    category: RuntimePluginAvailabilityCategory,
    reason: RuntimePluginAvailabilityReason,
) -> RuntimePluginAvailabilityRow<'a> {
    RuntimePluginAvailabilityRow {
        package_id: Some(descriptor.package_id),
        runtime_id: descriptor.runtime_id.clone(),
        required,
        maturity: descriptor.maturity,
        category,
        reason,
    }
}

pub(super) fn row_from_runtime(
    runtime_id: RuntimePluginId,
    required: bool,
    maturity: PluginMaturity,
    category: RuntimePluginAvailabilityCategory,
    reason: RuntimePluginAvailabilityReason,
) -> RuntimePluginAvailabilityRow<'static> {
    RuntimePluginAvailabilityRow {
        package_id: None,
        runtime_id,
        required,
        maturity,
        category,
        reason,
    }
}

impl RuntimePluginAvailabilitySummary {
    pub const fn row_count(self) -> usize {
        self.row_count
    }

    pub const fn missing_required_count(self) -> usize {
        self.missing_required_count
    }

    pub const fn category_count(self, category: RuntimePluginAvailabilityCategory) -> usize {
        match category {
            RuntimePluginAvailabilityCategory::MissingRequired => self.missing_required_count,
            category => self.category_counts[primary_category_index(category)],
        }
    }
}

pub(super) struct RuntimePluginAvailabilityGenerationBuilder<'a> {
    rows: Vec<RuntimePluginAvailabilityRow<'a>>,
    categories: [Vec<usize>; PRIMARY_AVAILABILITY_CATEGORY_COUNT],
    missing_required: Vec<usize>,
    by_runtime_id: HashMap<RuntimePluginId, usize>,
}

impl<'a> RuntimePluginAvailabilityGenerationBuilder<'a> {
    pub(super) fn new() -> Self {
        Self {
            rows: Vec::new(),
            categories: std::array::from_fn(|_| Vec::new()),
            missing_required: Vec::new(),
            by_runtime_id: HashMap::new(),
        }
    }

    pub(super) fn push(&mut self, missing_required: bool, row: RuntimePluginAvailabilityRow<'a>) {
        let row_index = self.rows.len();
        self.by_runtime_id.insert(row.runtime_id.clone(), row_index);
        self.categories[primary_category_index(row.category)].push(row_index);
        self.rows.push(row);
        if missing_required {
            self.missing_required.push(row_index);
        }
    }

    pub(super) fn finish(self) -> RuntimePluginAvailabilityGeneration<'a> {
        let category_counts = std::array::from_fn(|index| self.categories[index].len());
        let summary = RuntimePluginAvailabilitySummary {
            row_count: self.rows.len(),
            category_counts,
            missing_required_count: self.missing_required.len(),
        };
        RuntimePluginAvailabilityGeneration {
            rows: self.rows.into_boxed_slice(),
            categories: self.categories.map(Vec::into_boxed_slice),
            missing_required: self.missing_required.into_boxed_slice(),
            by_runtime_id: self.by_runtime_id,
            summary,
        }
    }
}
