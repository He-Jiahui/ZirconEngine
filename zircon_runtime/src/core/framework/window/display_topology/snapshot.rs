use std::collections::HashMap;
use std::num::{NonZeroU32, NonZeroU64};

use super::{
    DisplayId, DisplayLogicalInsets, DisplayLogicalRect, DisplayOrientation,
    DisplayOutputCapabilities, DisplayPhysicalRect, DisplayTopologyError,
};

/// A nonzero publication generation for a host's immutable display topology.
/// The publishing host advances it on each replacement so consumers can reject
/// stale placement or surface requests after a hotplug or mode change.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DisplayTopologyGeneration(NonZeroU64);

impl DisplayTopologyGeneration {
    pub(crate) const fn initial() -> Self {
        Self(NonZeroU64::MIN)
    }

    pub const fn new(raw: u64) -> Option<Self> {
        match NonZeroU64::new(raw) {
            Some(raw) => Some(Self(raw)),
            None => None,
        }
    }

    pub const fn get(self) -> u64 {
        self.0.get()
    }
}

/// Fully observed facts for one display in one topology generation.
#[derive(Clone, Debug, PartialEq)]
pub struct DisplaySnapshot {
    id: DisplayId,
    physical_bounds: DisplayPhysicalRect,
    usable_logical_bounds: DisplayLogicalRect,
    scale_factor: f64,
    refresh_rate_millihertz: Option<NonZeroU32>,
    orientation: DisplayOrientation,
    safe_area: Option<DisplayLogicalInsets>,
    output_capabilities: DisplayOutputCapabilities,
}

/// Backend-observed display state used to create a validated snapshot entry.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DisplayObservation {
    pub physical_bounds: DisplayPhysicalRect,
    pub usable_logical_bounds: DisplayLogicalRect,
    pub scale_factor: f64,
    pub refresh_rate_millihertz: Option<NonZeroU32>,
    pub orientation: DisplayOrientation,
    pub safe_area: Option<DisplayLogicalInsets>,
    pub output_capabilities: DisplayOutputCapabilities,
}

impl DisplaySnapshot {
    pub fn new(id: DisplayId, observed: DisplayObservation) -> Result<Self, DisplayTopologyError> {
        validate_scale_factor(&id, observed.scale_factor)?;
        validate_safe_area(&id, observed.usable_logical_bounds, observed.safe_area)?;
        Ok(Self {
            id,
            physical_bounds: observed.physical_bounds,
            usable_logical_bounds: observed.usable_logical_bounds,
            scale_factor: observed.scale_factor,
            refresh_rate_millihertz: observed.refresh_rate_millihertz,
            orientation: observed.orientation,
            safe_area: observed.safe_area,
            output_capabilities: observed.output_capabilities,
        })
    }

    pub fn id(&self) -> &DisplayId {
        &self.id
    }

    pub const fn physical_bounds(&self) -> DisplayPhysicalRect {
        self.physical_bounds
    }

    pub const fn usable_logical_bounds(&self) -> DisplayLogicalRect {
        self.usable_logical_bounds
    }

    pub const fn scale_factor(&self) -> f64 {
        self.scale_factor
    }

    pub const fn refresh_rate_millihertz(&self) -> Option<u32> {
        match self.refresh_rate_millihertz {
            Some(refresh_rate) => Some(refresh_rate.get()),
            None => None,
        }
    }

    pub const fn orientation(&self) -> DisplayOrientation {
        self.orientation
    }

    pub const fn safe_area(&self) -> Option<DisplayLogicalInsets> {
        self.safe_area
    }

    pub const fn output_capabilities(&self) -> DisplayOutputCapabilities {
        self.output_capabilities
    }
}

/// Immutable display topology published by a platform-host generation.
///
/// Construction validates all display identities once, then retains a private
/// ID-to-index table so primary and placement consumers do not linearly scan
/// monitor lists on frame or command hot paths.
#[derive(Clone, Debug)]
pub struct DisplayTopologySnapshot {
    generation: DisplayTopologyGeneration,
    displays: Vec<DisplaySnapshot>,
    indices_by_id: HashMap<DisplayId, usize>,
    primary_display: Option<DisplayId>,
}

impl DisplayTopologySnapshot {
    pub(crate) fn empty(generation: DisplayTopologyGeneration) -> Self {
        Self {
            generation,
            displays: Vec::new(),
            indices_by_id: HashMap::new(),
            primary_display: None,
        }
    }

    pub fn new(
        generation: DisplayTopologyGeneration,
        displays: Vec<DisplaySnapshot>,
        primary_display: Option<DisplayId>,
    ) -> Result<Self, DisplayTopologyError> {
        let mut indices_by_id = HashMap::new();
        indices_by_id
            .try_reserve(displays.len())
            .map_err(|_| DisplayTopologyError::CapacityExhausted)?;
        for (index, display) in displays.iter().enumerate() {
            if indices_by_id.insert(display.id.clone(), index).is_some() {
                return Err(DisplayTopologyError::DuplicateDisplay {
                    display: display.id.clone(),
                });
            }
        }
        if let Some(primary_display) = primary_display.as_ref() {
            if !indices_by_id.contains_key(primary_display) {
                return Err(DisplayTopologyError::UnknownPrimaryDisplay {
                    display: primary_display.clone(),
                });
            }
        }
        Ok(Self {
            generation,
            displays,
            indices_by_id,
            primary_display,
        })
    }

    pub const fn generation(&self) -> DisplayTopologyGeneration {
        self.generation
    }

    pub const fn len(&self) -> usize {
        self.displays.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.displays.is_empty()
    }

    pub fn primary_display_id(&self) -> Option<&DisplayId> {
        self.primary_display.as_ref()
    }

    pub fn primary_display(&self) -> Option<&DisplaySnapshot> {
        self.primary_display
            .as_ref()
            .and_then(|display| self.get(display))
    }

    pub fn get(&self, id: &DisplayId) -> Option<&DisplaySnapshot> {
        self.indices_by_id
            .get(id)
            .and_then(|index| self.displays.get(*index))
    }

    pub fn contains(&self, id: &DisplayId) -> bool {
        self.indices_by_id.contains_key(id)
    }

    pub fn displays(&self) -> impl ExactSizeIterator<Item = &DisplaySnapshot> {
        self.displays.iter()
    }
}

fn validate_scale_factor(id: &DisplayId, scale_factor: f64) -> Result<(), DisplayTopologyError> {
    if !scale_factor.is_finite() {
        return Err(DisplayTopologyError::NonFiniteScaleFactor {
            display: id.clone(),
            scale_factor,
        });
    }
    if scale_factor <= 0.0 {
        return Err(DisplayTopologyError::NonPositiveScaleFactor {
            display: id.clone(),
            scale_factor,
        });
    }
    Ok(())
}

fn validate_safe_area(
    id: &DisplayId,
    usable_bounds: DisplayLogicalRect,
    safe_area: Option<DisplayLogicalInsets>,
) -> Result<(), DisplayTopologyError> {
    let Some(safe_area) = safe_area else {
        return Ok(());
    };
    if safe_area.left() + safe_area.right() > usable_bounds.width()
        || safe_area.top() + safe_area.bottom() > usable_bounds.height()
    {
        return Err(DisplayTopologyError::SafeAreaExceedsUsableBounds {
            display: id.clone(),
        });
    }
    Ok(())
}
