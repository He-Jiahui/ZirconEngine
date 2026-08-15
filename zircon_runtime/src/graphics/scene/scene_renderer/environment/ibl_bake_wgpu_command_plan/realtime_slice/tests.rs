use crate::core::framework::render::{
    IblBakeArtifactContents, IblBakeArtifactRequest, ProceduralSkyParams,
};

use super::*;

#[test]
fn realtime_prefilter_slice_limits_faces_and_serializes_face_offset() {
    let request = request();
    let command = ibl_bake_wgpu_prefilter_command_for_slice(
        &request,
        RealtimeIblPrefilterDispatchSlice {
            mip_level: 3,
            first_face: 2,
            face_count: 2,
        },
    )
    .expect("valid PMREM slice");

    assert_eq!(
        command.kind,
        IblBakeComputeKernelKind::Pmrem { mip_level: 3 }
    );
    assert_eq!(command.dispatch_groups, [2, 2, 2]);
    assert_eq!(command.params.words()[5], 2);
    assert!(command.readback_copies.is_empty());
    assert!(command
        .wgsl_source
        .contains("params.first_face + global_id.z"));
}

#[test]
fn realtime_terminal_pmrem_slice_uses_single_dispatch_only_for_all_faces() {
    let request = request();
    let complete = ibl_bake_wgpu_prefilter_command_for_slice(
        &request,
        RealtimeIblPrefilterDispatchSlice {
            mip_level: 7,
            first_face: 0,
            face_count: 6,
        },
    )
    .expect("complete terminal PMREM slice");
    let partial = ibl_bake_wgpu_prefilter_command_for_slice(
        &request,
        RealtimeIblPrefilterDispatchSlice {
            mip_level: 7,
            first_face: 2,
            face_count: 2,
        },
    )
    .expect("partial terminal PMREM slice");

    assert_eq!(complete.dispatch_groups, [1, 1, 1]);
    assert_eq!(complete.params.words()[7], 1.0_f32.to_bits());
    assert_eq!(partial.dispatch_groups, [1, 1, 2]);
    assert_eq!(partial.params.words()[5], 2);
    assert_eq!(partial.params.words()[7], 0);
}

#[test]
fn realtime_prefilter_slice_rejects_invalid_mips_and_face_ranges() {
    let request = request();
    for slice in [
        RealtimeIblPrefilterDispatchSlice {
            mip_level: 8,
            first_face: 0,
            face_count: 6,
        },
        RealtimeIblPrefilterDispatchSlice {
            mip_level: 0,
            first_face: 5,
            face_count: 2,
        },
        RealtimeIblPrefilterDispatchSlice {
            mip_level: 0,
            first_face: 0,
            face_count: 0,
        },
    ] {
        assert!(ibl_bake_wgpu_prefilter_command_for_slice(&request, slice).is_none());
    }
}

fn request() -> IblBakeArtifactRequest {
    IblBakeArtifactRequest::new(
        ProceduralSkyParams::default_gradient().ibl_bake_key(),
        128,
        8,
    )
    .with_required_contents(IblBakeArtifactContents::PMREM)
}
