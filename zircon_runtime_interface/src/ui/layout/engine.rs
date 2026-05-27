use std::collections::BTreeMap;

use serde::{Deserialize, Deserializer, Serialize};

use crate::ui::event_ui::UiNodeId;

use super::UiContainerKind;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiLayoutEngineBackend {
    LegacyZircon,
    Taffy,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiLayoutEngineFamily {
    Free,
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
                | Self::Container
                | Self::Overlay
                | Self::Scrollable
                | Self::VirtualizedList
                | Self::Masonry
        )
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
        Self::legacy_zircon()
    }
}

impl UiLayoutEngineCapability {
    pub fn legacy_zircon() -> Self {
        Self {
            backend: UiLayoutEngineBackend::LegacyZircon,
            supported_families: vec![
                UiLayoutEngineFamily::Free,
                UiLayoutEngineFamily::Container,
                UiLayoutEngineFamily::Overlay,
                UiLayoutEngineFamily::Flex,
                UiLayoutEngineFamily::Grid,
                UiLayoutEngineFamily::Scrollable,
                UiLayoutEngineFamily::Wrap,
                UiLayoutEngineFamily::Masonry,
                UiLayoutEngineFamily::VirtualizedList,
            ],
            supports_content_measure: true,
            supports_dpi_scaling: true,
        }
    }

    pub fn taffy_flex_grid_block() -> Self {
        Self {
            backend: UiLayoutEngineBackend::Taffy,
            supported_families: vec![
                UiLayoutEngineFamily::Flex,
                UiLayoutEngineFamily::Grid,
                UiLayoutEngineFamily::Block,
                UiLayoutEngineFamily::Wrap,
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
        let family = match container {
            UiContainerKind::Free => UiLayoutEngineFamily::Free,
            UiContainerKind::Container | UiContainerKind::Space | UiContainerKind::SizeBox(_) => {
                UiLayoutEngineFamily::Container
            }
            UiContainerKind::Overlay => UiLayoutEngineFamily::Overlay,
            UiContainerKind::HorizontalBox(_) | UiContainerKind::VerticalBox(_) => {
                UiLayoutEngineFamily::Flex
            }
            UiContainerKind::ScrollableBox(config) => {
                if config.virtualization.is_some() {
                    UiLayoutEngineFamily::VirtualizedList
                } else {
                    UiLayoutEngineFamily::Scrollable
                }
            }
            UiContainerKind::WrapBox(_) => UiLayoutEngineFamily::Wrap,
            UiContainerKind::GridBox(_) => UiLayoutEngineFamily::Grid,
            UiContainerKind::MasonryBox(_) => UiLayoutEngineFamily::Masonry,
        };
        Self::new(family)
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
            requested_backend: UiLayoutEngineBackend::LegacyZircon,
            selected_backend: UiLayoutEngineBackend::LegacyZircon,
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
                support: if fallback.supports_family(request.family) {
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

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct UiLayoutEngineSelectionReport {
    pub selections: Vec<UiLayoutEngineSelection>,
    pub request_count: u64,
    pub taffy_selected_count: u64,
    pub legacy_selected_count: u64,
    pub fallback_count: u64,
    pub unsupported_count: u64,
    pub fallback_reason_counts: Vec<UiLayoutEngineFallbackReasonCount>,
    pub taffy_tree_build_count: u64,
    pub taffy_tree_node_count: u64,
}

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
            legacy_selected_count: u64,
            fallback_count: u64,
            unsupported_count: u64,
            fallback_reason_counts: Vec<UiLayoutEngineFallbackReasonCount>,
            taffy_tree_build_count: u64,
            taffy_tree_node_count: u64,
        }

        let wire = WireReport::deserialize(deserializer)?;
        let mut report = Self {
            selections: wire.selections,
            request_count: wire.request_count,
            taffy_selected_count: wire.taffy_selected_count,
            legacy_selected_count: wire.legacy_selected_count,
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
            selections,
            ..Self::default()
        };
        report.recompute_counts();
        report
    }

    pub fn recompute_counts(&mut self) {
        self.request_count = self.selections.len() as u64;
        self.taffy_selected_count = 0;
        self.legacy_selected_count = 0;
        self.fallback_count = 0;
        self.unsupported_count = 0;
        self.taffy_tree_build_count = 0;
        self.taffy_tree_node_count = 0;
        self.fallback_reason_counts.clear();
        let mut fallback_reason_counts =
            BTreeMap::<Option<UiLayoutEngineFallbackReason>, u64>::new();

        for selection in &self.selections {
            match selection.selected_backend {
                UiLayoutEngineBackend::LegacyZircon => self.legacy_selected_count += 1,
                UiLayoutEngineBackend::Taffy => self.taffy_selected_count += 1,
            }
            match selection.support {
                UiLayoutEngineSupport::Native => {}
                UiLayoutEngineSupport::Fallback => self.fallback_count += 1,
                UiLayoutEngineSupport::Unsupported => self.unsupported_count += 1,
            }
            if selection.support != UiLayoutEngineSupport::Native {
                *fallback_reason_counts
                    .entry(selection.fallback_reason)
                    .or_default() += 1;
            }
            if let Some(stats) = selection.taffy_tree_build {
                self.taffy_tree_build_count = self
                    .taffy_tree_build_count
                    .saturating_add(stats.build_count);
                self.taffy_tree_node_count =
                    self.taffy_tree_node_count.saturating_add(stats.node_count);
            }
        }

        self.fallback_reason_counts = fallback_reason_counts
            .into_iter()
            .map(|(reason, count)| UiLayoutEngineFallbackReasonCount { reason, count })
            .collect();
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
