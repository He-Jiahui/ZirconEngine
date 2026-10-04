//! Shared semantic icon vocabulary used by UI producers and render consumers.
//!
//! This module contains only deterministic value parsing and sizing. Rasterization
//! remains owned by the runtime UI/graphics boundary.

/// Semantic icon size tiers for deterministic native vector painters.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiIconSize {
    Small,
    Medium,
    Large,
    ExtraLarge,
}

impl UiIconSize {
    pub fn parse(size: &str) -> Option<Self> {
        match size.trim().to_ascii_lowercase().as_str() {
            "s" | "small" => Some(Self::Small),
            "m" | "medium" => Some(Self::Medium),
            "l" | "large" => Some(Self::Large),
            "xl" | "extra-large" | "extra_large" => Some(Self::ExtraLarge),
            _ => None,
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Small => "s",
            Self::Medium => "m",
            Self::Large => "l",
            Self::ExtraLarge => "xl",
        }
    }

    pub const fn logical_extent(self) -> f32 {
        match self {
            Self::Small => 16.0,
            Self::Medium => 20.0,
            Self::Large => 24.0,
            Self::ExtraLarge => 32.0,
        }
    }

    pub fn nearest_for(available: f32) -> Self {
        match available {
            value if value <= 18.0 => Self::Small,
            value if value <= 22.0 => Self::Medium,
            value if value <= 28.0 => Self::Large,
            _ => Self::ExtraLarge,
        }
    }
}

/// Built-in icon identities understood by the native vector painter.
///
/// Project-provided `UiIconAsset` resources remain a separate asset-backed path.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiBuiltinIcon {
    AlertTriangle,
    ArrowUp,
    Check,
    CheckCircle,
    ChevronDown,
    ChevronLeft,
    ChevronRight,
    ChevronUp,
    Close,
    Info,
    Minus,
    MoreHorizontal,
    MoreVertical,
    Package,
    Plus,
    Search,
    X,
    XCircle,
}

impl UiBuiltinIcon {
    pub fn parse(icon: &str) -> Option<Self> {
        UiBuiltinIconReference::parse(icon).map(UiBuiltinIconReference::icon)
    }

    fn parse_id(icon: &str) -> Option<Self> {
        match icon.trim().to_ascii_lowercase().as_str() {
            "alert-triangle" => Some(Self::AlertTriangle),
            "arrow-up" => Some(Self::ArrowUp),
            "check" => Some(Self::Check),
            "check-circle" => Some(Self::CheckCircle),
            "chevron-down" => Some(Self::ChevronDown),
            "chevron-left" => Some(Self::ChevronLeft),
            "chevron-right" => Some(Self::ChevronRight),
            "chevron-up" => Some(Self::ChevronUp),
            "close" => Some(Self::Close),
            "info" => Some(Self::Info),
            "minus" => Some(Self::Minus),
            "more-horizontal" => Some(Self::MoreHorizontal),
            "more-vertical" => Some(Self::MoreVertical),
            "package" => Some(Self::Package),
            "plus" => Some(Self::Plus),
            "search" => Some(Self::Search),
            "x" => Some(Self::X),
            "x-circle" => Some(Self::XCircle),
            _ => None,
        }
    }
}

/// A parsed built-in icon identity with an optional semantic size override.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UiBuiltinIconReference {
    icon: UiBuiltinIcon,
    size: Option<UiIconSize>,
}

impl UiBuiltinIconReference {
    pub fn parse(reference: &str) -> Option<Self> {
        let reference = reference.trim();
        let (icon, size) = if let Some((icon, size)) = reference.rsplit_once('@') {
            (icon, Some(UiIconSize::parse(size)?))
        } else {
            (reference, None)
        };
        Some(Self {
            icon: UiBuiltinIcon::parse_id(icon)?,
            size,
        })
    }

    pub const fn icon(self) -> UiBuiltinIcon {
        self.icon
    }

    pub fn resolved_size(self, available: f32) -> UiIconSize {
        self.size
            .unwrap_or_else(|| UiIconSize::nearest_for(available))
    }
}

pub fn builtin_icon_supported(icon: &str) -> bool {
    UiBuiltinIcon::parse(icon).is_some()
}

#[cfg(test)]
#[path = "tests/icon.rs"]
mod tests;
