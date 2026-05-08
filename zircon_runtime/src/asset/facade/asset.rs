use crate::core::resource::{ResourceData, ResourceMarker};

/// Typed asset contract layered over the existing resource marker model.
pub trait Asset: ResourceData + Clone + 'static {
    type Marker: ResourceMarker;

    const LABEL: &'static str;
}
