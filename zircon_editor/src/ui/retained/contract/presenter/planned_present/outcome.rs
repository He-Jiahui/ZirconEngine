use super::super::super::super::super::data::FrameRect;
use super::super::super::backbuffer::RepaintOutcome;
use super::super::super::surface_io::damage_pixel_count;

pub(super) fn repaint_outcome_for_damage(
    damage: Option<FrameRect>,
    size: (u32, u32),
) -> RepaintOutcome {
    if let Some(damage) = damage {
        return RepaintOutcome {
            painted_pixels: damage_pixel_count(&damage, size),
            damage: Some(damage),
            full_paint: false,
            region_paint: true,
        };
    }

    RepaintOutcome {
        damage: None,
        painted_pixels: (size.0 as u64) * (size.1 as u64),
        full_paint: true,
        region_paint: false,
    }
}
