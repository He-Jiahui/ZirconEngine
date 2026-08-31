use super::super::super::data::HostWindowPresentationData;
use super::super::super::profiling_hit_routes::route_contains_profile_frame;
use super::super::{UiProfileHitSample, UiProfileNamedFrame, UiProfilePoint};
use super::frame_math::profile_frame_center;

#[cfg(test)]
mod capacity_tests;

const PROFILE_HIT_SAMPLES_PER_FRAME: usize = 3;

pub(in crate::ui::retained_host::host_contract) fn collect_hit_samples(
    frames: &[UiProfileNamedFrame],
    presentation: &HostWindowPresentationData,
) -> Vec<UiProfileHitSample> {
    let mut samples = Vec::with_capacity(profile_hit_sample_capacity(frames.len()));
    for frame in frames {
        samples.extend(hit_samples_for_frame(frame, presentation));
    }
    samples
}

pub(in crate::ui::retained_host::host_contract) fn hit_samples_for_frame(
    frame: &UiProfileNamedFrame,
    presentation: &HostWindowPresentationData,
) -> Vec<UiProfileHitSample> {
    let mut samples = Vec::with_capacity(PROFILE_HIT_SAMPLES_PER_FRAME);
    let center = profile_frame_center(&frame.frame);
    samples.push(UiProfileHitSample {
        id: frame.id.clone(),
        kind: frame.kind.clone(),
        surface: frame.surface.clone(),
        sample: "center".to_string(),
        expected_hit: true,
        route_hit: profile_route_hit(presentation, frame, &center),
        point: center,
    });
    let outside_left = UiProfilePoint {
        x: frame.frame.x - 3.0,
        y: frame.frame.y + frame.frame.height * 0.5,
    };
    samples.push(UiProfileHitSample {
        id: frame.id.clone(),
        kind: frame.kind.clone(),
        surface: frame.surface.clone(),
        sample: "outside_left".to_string(),
        expected_hit: false,
        route_hit: profile_route_hit(presentation, frame, &outside_left),
        point: outside_left,
    });
    let outside_bottom = UiProfilePoint {
        x: frame.frame.x + frame.frame.width * 0.5,
        y: frame.frame.y + frame.frame.height + 3.0,
    };
    samples.push(UiProfileHitSample {
        id: frame.id.clone(),
        kind: frame.kind.clone(),
        surface: frame.surface.clone(),
        sample: "outside_bottom".to_string(),
        expected_hit: false,
        route_hit: profile_route_hit(presentation, frame, &outside_bottom),
        point: outside_bottom,
    });
    samples
}

fn profile_hit_sample_capacity(frame_count: usize) -> usize {
    frame_count.saturating_mul(PROFILE_HIT_SAMPLES_PER_FRAME)
}

fn profile_route_hit(
    presentation: &HostWindowPresentationData,
    frame: &UiProfileNamedFrame,
    point: &UiProfilePoint,
) -> bool {
    route_contains_profile_frame(
        presentation,
        frame.kind.as_str(),
        frame.id.as_str(),
        frame.surface.as_str(),
        point.x,
        point.y,
    )
}
