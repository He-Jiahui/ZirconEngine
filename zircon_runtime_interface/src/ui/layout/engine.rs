use serde::{Deserialize, Deserializer, Serialize};

use crate::ui::event_ui::UiNodeId;
use crate::ui::surface::{UiPersistentSequence, UiPersistentSequenceCowStats};

use super::UiContainerKind;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiLayoutEngineBackend {
    Zircon,
    Taffy,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiLayoutEngineFamily {
    Free,
    Canvas,
    Container,
    Overlay,
    Flex,
    Grid,
    Block,
    Scrollable,
    Wrap,
    Masonry,
    VirtualizedList,
}

impl UiLayoutEngineFamily {
    pub const fn is_zircon_owned(self) -> bool {
        matches!(
            self,
            Self::Free
                | Self::Canvas
                | Self::Container
                | Self::Overlay
                | Self::Scrollable
                | Self::VirtualizedList
                | Self::Masonry
        )
    }

    pub const fn is_taffy_owned(self) -> bool {
        matches!(self, Self::Flex | Self::Grid | Self::Block | Self::Wrap)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiLayoutEngineCapability {
    pub backend: UiLayoutEngineBackend,
    pub supported_families: Vec<UiLayoutEngineFamily>,
    pub supports_content_measure: bool,
    pub supports_dpi_scaling: bool,
}

impl Default for UiLayoutEngineCapability {
    fn default() -> Self {
        Self::zircon()
    }
}

impl UiLayoutEngineCapability {
    pub fn zircon() -> Self {
        Self {
            backend: UiLayoutEngineBackend::Zircon,
            supported_families: vec![
                UiLayoutEngineFamily::Free,
                UiLayoutEngineFamily::Canvas,
                UiLayoutEngineFamily::Container,
                UiLayoutEngineFamily::Overlay,
                UiLayoutEngineFamily::Flex,
                UiLayoutEngineFamily::Grid,
                UiLayoutEngineFamily::Block,
                UiLayoutEngineFamily::Scrollable,
                UiLayoutEngineFamily::Wrap,
                UiLayoutEngineFamily::Masonry,
                UiLayoutEngineFamily::VirtualizedList,
            ],
            supports_content_measure: true,
            supports_dpi_scaling: true,
        }
    }

    pub fn taffy_flex_grid_wrap_block() -> Self {
        Self {
            backend: UiLayoutEngineBackend::Taffy,
            supported_families: vec![
                UiLayoutEngineFamily::Flex,
                UiLayoutEngineFamily::Grid,
                UiLayoutEngineFamily::Wrap,
                UiLayoutEngineFamily::Block,
            ],
            supports_content_measure: true,
            supports_dpi_scaling: true,
        }
    }

    pub fn supports_family(&self, family: UiLayoutEngineFamily) -> bool {
        self.supported_families.contains(&family)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiLayoutEngineRequest {
    pub family: UiLayoutEngineFamily,
    pub needs_content_measure: bool,
    pub needs_dpi_scaling: bool,
}

impl Default for UiLayoutEngineRequest {
    fn default() -> Self {
        Self::new(UiLayoutEngineFamily::Free)
    }
}

impl UiLayoutEngineRequest {
    pub const fn new(family: UiLayoutEngineFamily) -> Self {
        Self {
            family,
            needs_content_measure: false,
            needs_dpi_scaling: true,
        }
    }

    pub const fn requires_zircon_semantics(&self) -> bool {
        self.family.is_zircon_owned()
    }

    pub const fn from_container_kind(container: UiContainerKind) -> Self {
        Self::new(container.layout_engine_family())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiLayoutEngineSupport {
    Native,
    Fallback,
    Unsupported,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiLayoutEngineFallbackReason {
    UnsupportedFamily,
    MissingContentMeasure,
    MissingDpiScaling,
    ZirconOwnedSemantics,
    UnsupportedChildVisibility,
    ChildPlacementPolicy,
    AxisConstraintPriority,
    InvalidLayoutValue,
    SlotFramePolicy,
    SlotCanvasPlacement,
    TaffyStyleUnavailable,
    TaffyTreeBuildFailed,
    TaffyComputeFailed,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiLayoutEngineFallbackReasonCount {
    pub reason: Option<UiLayoutEngineFallbackReason>,
    pub count: u64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiLayoutEngineTaffyTreeBuildStats {
    pub build_count: u64,
    pub node_count: u64,
}

impl UiLayoutEngineTaffyTreeBuildStats {
    pub const fn new(node_count: u64) -> Self {
        Self {
            build_count: 1,
            node_count,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiLayoutEngineSelection {
    pub node_id: Option<UiNodeId>,
    pub request: UiLayoutEngineRequest,
    pub requested_backend: UiLayoutEngineBackend,
    pub selected_backend: UiLayoutEngineBackend,
    pub support: UiLayoutEngineSupport,
    pub fallback_reason: Option<UiLayoutEngineFallbackReason>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub taffy_tree_build: Option<UiLayoutEngineTaffyTreeBuildStats>,
}

impl Default for UiLayoutEngineSelection {
    fn default() -> Self {
        let request = UiLayoutEngineRequest::default();
        Self {
            node_id: None,
            request,
            requested_backend: UiLayoutEngineBackend::Zircon,
            selected_backend: UiLayoutEngineBackend::Zircon,
            support: UiLayoutEngineSupport::Native,
            fallback_reason: None,
            taffy_tree_build: None,
        }
    }
}

impl UiLayoutEngineSelection {
    pub fn select(
        request: &UiLayoutEngineRequest,
        preferred: &UiLayoutEngineCapability,
        fallback: &UiLayoutEngineCapability,
    ) -> Self {
        if let Some(reason) = unsupported_reason(request, preferred) {
            return Self {
                node_id: None,
                request: request.clone(),
                requested_backend: preferred.backend,
                selected_backend: fallback.backend,
                support: if unsupported_reason(request, fallback).is_none() {
                    UiLayoutEngineSupport::Fallback
                } else {
                    UiLayoutEngineSupport::Unsupported
                },
                fallback_reason: Some(reason),
                taffy_tree_build: None,
            };
        }

        Self {
            node_id: None,
            request: request.clone(),
            requested_backend: preferred.backend,
            selected_backend: preferred.backend,
            support: UiLayoutEngineSupport::Native,
            fallback_reason: None,
            taffy_tree_build: None,
        }
    }

    pub fn with_node_id(mut self, node_id: UiNodeId) -> Self {
        self.node_id = Some(node_id);
        self
    }

    pub fn with_taffy_tree_build(mut self, stats: UiLayoutEngineTaffyTreeBuildStats) -> Self {
        self.taffy_tree_build = Some(stats);
        self
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct UiLayoutEngineSelectionReport {
    pub selections: UiPersistentSequence<UiLayoutEngineSelection>,
    pub request_count: u64,
    pub taffy_selected_count: u64,
    pub zircon_selected_count: u64,
    pub fallback_count: u64,
    pub unsupported_count: u64,
    pub fallback_reason_counts: Vec<UiLayoutEngineFallbackReasonCount>,
    pub taffy_tree_build_count: u64,
    pub taffy_tree_node_count: u64,
}

impl Eq for UiLayoutEngineSelectionReport {}

impl<'de> Deserialize<'de> for UiLayoutEngineSelectionReport {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Default, Deserialize)]
        #[serde(default)]
        struct WireReport {
            selections: Vec<UiLayoutEngineSelection>,
            request_count: u64,
            taffy_selected_count: u64,
            zircon_selected_count: u64,
            fallback_count: u64,
            unsupported_count: u64,
            fallback_reason_counts: Vec<UiLayoutEngineFallbackReasonCount>,
            taffy_tree_build_count: u64,
            taffy_tree_node_count: u64,
        }

        let wire = WireReport::deserialize(deserializer)?;
        let mut report = Self {
            selections: wire.selections.into(),
            request_count: wire.request_count,
            taffy_selected_count: wire.taffy_selected_count,
            zircon_selected_count: wire.zircon_selected_count,
            fallback_count: wire.fallback_count,
            unsupported_count: wire.unsupported_count,
            fallback_reason_counts: wire.fallback_reason_counts,
            taffy_tree_build_count: wire.taffy_tree_build_count,
            taffy_tree_node_count: wire.taffy_tree_node_count,
        };
        report.recompute_counts();
        Ok(report)
    }
}

impl UiLayoutEngineSelectionReport {
    pub fn from_selections(selections: Vec<UiLayoutEngineSelection>) -> Self {
        let mut report = Self {
            selections: selections.into(),
            ..Self::default()
        };
        report.recompute_counts();
        report
    }

    pub fn recompute_counts(&mut self) {
        self.request_count = self.selections.len() as u64;
        self.taffy_selected_count = 0;
        self.zircon_selected_count = 0;
        self.fallback_count = 0;
        self.unsupported_count = 0;
        self.taffy_tree_build_count = 0;
        self.taffy_tree_node_count = 0;
        self.fallback_reason_counts.clear();

        for selection in &self.selections {
            match selection.selected_backend {
                UiLayoutEngineBackend::Zircon => self.zircon_selected_count += 1,
                UiLayoutEngineBackend::Taffy => self.taffy_selected_count += 1,
            }
            match selection.support {
                UiLayoutEngineSupport::Native => {}
                UiLayoutEngineSupport::Fallback => self.fallback_count += 1,
                UiLayoutEngineSupport::Unsupported => self.unsupported_count += 1,
            }
            if selection.support != UiLayoutEngineSupport::Native {
                Self::increment_fallback_reason_count(
                    &mut self.fallback_reason_counts,
                    selection.fallback_reason,
                );
            }
            if let Some(stats) = selection.taffy_tree_build {
                self.taffy_tree_build_count = self
                    .taffy_tree_build_count
                    .saturating_add(stats.build_count);
                self.taffy_tree_node_count =
                    self.taffy_tree_node_count.saturating_add(stats.node_count);
            }
        }
    }

    fn increment_fallback_reason_count(
        fallback_reason_counts: &mut Vec<UiLayoutEngineFallbackReasonCount>,
        reason: Option<UiLayoutEngineFallbackReason>,
    ) {
        match fallback_reason_counts.binary_search_by_key(&reason, |entry| entry.reason) {
            Ok(index) => fallback_reason_counts[index].count += 1,
            Err(index) => fallback_reason_counts.insert(
                index,
                UiLayoutEngineFallbackReasonCount { reason, count: 1 },
            ),
        }
    }

    /// Replaces one stable node route while maintaining aggregate diagnostics in place.
    ///
    /// Returns `false` when the report is internally inconsistent or a saturated aggregate
    /// cannot be reversed safely; callers can then rebuild the report conservatively.
    pub fn replace_selection_at(
        &mut self,
        index: usize,
        replacement: UiLayoutEngineSelection,
    ) -> bool {
        self.replace_selection_at_with_cow_stats(index, replacement)
            .is_some()
    }

    /// Replaces one stable node route and reports the persistent storage copied on write.
    ///
    /// Returns `None` under the same conditions that make [`Self::replace_selection_at`]
    /// return `false`. An unchanged replacement succeeds with zero copy-on-write work.
    pub fn replace_selection_at_with_cow_stats(
        &mut self,
        index: usize,
        replacement: UiLayoutEngineSelection,
    ) -> Option<UiPersistentSequenceCowStats> {
        let Some(previous) = self.selections.get(index) else {
            return None;
        };
        if previous == &replacement {
            return Some(UiPersistentSequenceCowStats::default());
        }
        let previous_selected_backend = previous.selected_backend;
        let previous_support = previous.support;
        let previous_fallback_reason = previous.fallback_reason;
        let previous_taffy_tree_build = previous.taffy_tree_build;

        let selected_count = match previous_selected_backend {
            UiLayoutEngineBackend::Zircon => self.zircon_selected_count,
            UiLayoutEngineBackend::Taffy => self.taffy_selected_count,
        };
        if selected_count == 0 {
            return None;
        }
        let support_count = match previous_support {
            UiLayoutEngineSupport::Native => None,
            UiLayoutEngineSupport::Fallback => Some(self.fallback_count),
            UiLayoutEngineSupport::Unsupported => Some(self.unsupported_count),
        };
        if support_count.is_some_and(|count| count == 0) {
            return None;
        }
        let previous_reason_index = if previous_support == UiLayoutEngineSupport::Native {
            None
        } else {
            let Some(index) = self
                .fallback_reason_counts
                .iter()
                .position(|entry| entry.reason == previous_fallback_reason && entry.count > 0)
            else {
                return None;
            };
            Some(index)
        };
        if let Some(stats) = previous_taffy_tree_build {
            if self.taffy_tree_build_count < stats.build_count
                || self.taffy_tree_node_count < stats.node_count
                || (stats.build_count > 0 && self.taffy_tree_build_count == u64::MAX)
                || (stats.node_count > 0 && self.taffy_tree_node_count == u64::MAX)
            {
                return None;
            }
        }

        match previous_selected_backend {
            UiLayoutEngineBackend::Zircon => self.zircon_selected_count -= 1,
            UiLayoutEngineBackend::Taffy => self.taffy_selected_count -= 1,
        }
        match previous_support {
            UiLayoutEngineSupport::Native => {}
            UiLayoutEngineSupport::Fallback => self.fallback_count -= 1,
            UiLayoutEngineSupport::Unsupported => self.unsupported_count -= 1,
        }
        if let Some(index) = previous_reason_index {
            self.fallback_reason_counts[index].count -= 1;
            if self.fallback_reason_counts[index].count == 0 {
                self.fallback_reason_counts.remove(index);
            }
        }
        if let Some(stats) = previous_taffy_tree_build {
            self.taffy_tree_build_count -= stats.build_count;
            self.taffy_tree_node_count -= stats.node_count;
        }

        match replacement.selected_backend {
            UiLayoutEngineBackend::Zircon => self.zircon_selected_count += 1,
            UiLayoutEngineBackend::Taffy => self.taffy_selected_count += 1,
        }
        match replacement.support {
            UiLayoutEngineSupport::Native => {}
            UiLayoutEngineSupport::Fallback => self.fallback_count += 1,
            UiLayoutEngineSupport::Unsupported => self.unsupported_count += 1,
        }
        if replacement.support != UiLayoutEngineSupport::Native {
            Self::increment_fallback_reason_count(
                &mut self.fallback_reason_counts,
                replacement.fallback_reason,
            );
        }
        if let Some(stats) = replacement.taffy_tree_build {
            self.taffy_tree_build_count = self
                .taffy_tree_build_count
                .saturating_add(stats.build_count);
            self.taffy_tree_node_count =
                self.taffy_tree_node_count.saturating_add(stats.node_count);
        }
        let (selection, cow_stats) = self
            .selections
            .get_mut_with_stats(index)
            .expect("validated layout selection index must remain present");
        *selection = replacement;
        Some(cow_stats)
    }
}

fn unsupported_reason(
    request: &UiLayoutEngineRequest,
    capability: &UiLayoutEngineCapability,
) -> Option<UiLayoutEngineFallbackReason> {
    if capability.backend == UiLayoutEngineBackend::Taffy && request.requires_zircon_semantics() {
        return Some(UiLayoutEngineFallbackReason::ZirconOwnedSemantics);
    }
    if !capability.supports_family(request.family) {
        return Some(UiLayoutEngineFallbackReason::UnsupportedFamily);
    }
    if request.needs_content_measure && !capability.supports_content_measure {
        return Some(UiLayoutEngineFallbackReason::MissingContentMeasure);
    }
    if request.needs_dpi_scaling && !capability.supports_dpi_scaling {
        return Some(UiLayoutEngineFallbackReason::MissingDpiScaling);
    }
    None
}
