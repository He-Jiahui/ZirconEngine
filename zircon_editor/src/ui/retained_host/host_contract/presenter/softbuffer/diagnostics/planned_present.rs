mod model;
mod outcome;

use self::model::planned_present;
pub(in crate::ui::retained_host::host_contract) use self::model::PlannedPresent;
use self::outcome::repaint_outcome_for_damage;
use super::super::super::super::data::{FrameRect, HostWindowPresentationData};
use super::super::super::super::diagnostics::{
    HostInvalidationDiagnostics, HostRefreshDiagnostics,
};
use super::super::surface_io::pixel_bounds;
use super::overlay::damage_with_debug_overlay;

pub(in crate::ui::retained_host::host_contract) fn plan_present_for_diagnostics(
    current: &HostRefreshDiagnostics,
    can_region_repaint: bool,
    last_debug_overlay_text: Option<&str>,
    presentation: &HostWindowPresentationData,
    damage: Option<FrameRect>,
    invalidation: HostInvalidationDiagnostics,
    size: (u32, u32),
) -> PlannedPresent {
    let mut damage = if can_region_repaint
        && damage
            .as_ref()
            .is_some_and(|damage| pixel_bounds(damage, size).is_some())
    {
        damage
    } else {
        None
    };

    // The overlay text includes painted pixels, and text width can expand damage.
    // Iterate until overlay text and damage describe the same present frame.
    for _ in 0..8 {
        let outcome = repaint_outcome_for_damage(damage.clone(), size);
        let mut diagnostics = current.clone();
        diagnostics.record_present(
            outcome.painted_pixels,
            outcome.full_paint,
            outcome.region_paint,
        );
        let overlay_text = diagnostics
            .clone()
            .with_invalidation_diagnostics(invalidation)
            .overlay_text();
        let expanded_damage = if outcome.region_paint {
            damage_with_debug_overlay(
                damage.clone(),
                last_debug_overlay_text,
                &overlay_text,
                size,
                presentation,
            )
        } else {
            None
        };
        if expanded_damage == damage {
            return planned_present(presentation, expanded_damage, diagnostics, overlay_text);
        }
        damage = expanded_damage;
    }

    let outcome = repaint_outcome_for_damage(damage.clone(), size);
    let mut diagnostics = current.clone();
    diagnostics.record_present(
        outcome.painted_pixels,
        outcome.full_paint,
        outcome.region_paint,
    );
    let overlay_text = diagnostics
        .clone()
        .with_invalidation_diagnostics(invalidation)
        .overlay_text();
    planned_present(presentation, damage, diagnostics, overlay_text)
}
