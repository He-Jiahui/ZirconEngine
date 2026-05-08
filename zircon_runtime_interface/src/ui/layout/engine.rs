use serde::{Deserialize, Serialize};

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
    VirtualizedList,
}

impl UiLayoutEngineFamily {
    pub const fn is_zircon_owned(self) -> bool {
        matches!(
            self,
            Self::Free | Self::Container | Self::Overlay | Self::Scrollable | Self::VirtualizedList
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
            UiContainerKind::Container | UiContainerKind::Space => UiLayoutEngineFamily::Container,
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
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiLayoutEngineSelection {
    pub request: UiLayoutEngineRequest,
    pub requested_backend: UiLayoutEngineBackend,
    pub selected_backend: UiLayoutEngineBackend,
    pub support: UiLayoutEngineSupport,
    pub fallback_reason: Option<UiLayoutEngineFallbackReason>,
}

impl Default for UiLayoutEngineSelection {
    fn default() -> Self {
        let request = UiLayoutEngineRequest::default();
        Self {
            request,
            requested_backend: UiLayoutEngineBackend::LegacyZircon,
            selected_backend: UiLayoutEngineBackend::LegacyZircon,
            support: UiLayoutEngineSupport::Native,
            fallback_reason: None,
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
                request: request.clone(),
                requested_backend: preferred.backend,
                selected_backend: fallback.backend,
                support: if fallback.supports_family(request.family) {
                    UiLayoutEngineSupport::Fallback
                } else {
                    UiLayoutEngineSupport::Unsupported
                },
                fallback_reason: Some(reason),
            };
        }

        Self {
            request: request.clone(),
            requested_backend: preferred.backend,
            selected_backend: preferred.backend,
            support: UiLayoutEngineSupport::Native,
            fallback_reason: None,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiLayoutEngineSelectionReport {
    pub selections: Vec<UiLayoutEngineSelection>,
    pub request_count: u64,
    pub taffy_selected_count: u64,
    pub legacy_selected_count: u64,
    pub fallback_count: u64,
    pub unsupported_count: u64,
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
        }
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
