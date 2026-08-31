use crate::core::framework::render::{
    IblBakeArtifactContents, IblBakeArtifactDescriptor, IblBakeArtifactRequest, ProceduralSkyParams,
};
use crate::graphics::scene::scene_renderer::environment::ibl_bake_shader_plan::{
    ibl_bake_irradiance_sh9_kernel_plan, ibl_bake_pmrem_kernel_plan, IblBakeComputeKernelKind,
};

use super::super::{
    ibl_bake_wgpu_command_plan_for_request, ibl_bake_wgpu_command_plan_for_runtime_kernel,
};
use super::*;

#[test]
fn realtime_pmrem_kernel_command_has_no_readback_description() {
    let request = request();
    let mut expected = ibl_bake_wgpu_command_plan_for_request(&request)
        .commands
        .into_iter()
        .find(|command| command.kind == IblBakeComputeKernelKind::Pmrem { mip_level: 3 })
        .expect("default PMREM request must contain mip three");
    expected.readback_copies.clear();

    let actual = ibl_bake_wgpu_command_plan_for_runtime_kernel(
        &request,
        IblBakeArtifactDescriptor::current_for_runtime_cache_request(&request),
        ibl_bake_pmrem_kernel_plan(&request, 3),
    );

    assert_eq!(actual, expected);
    assert!(actual.readback_copies.is_empty());
}

#[test]
fn realtime_sh9_kernel_command_matches_the_full_request_without_readback() {
    let request = request().with_required_contents(IblBakeArtifactContents::PMREM_SH9);
    let mut expected = ibl_bake_wgpu_command_plan_for_request(&request)
        .commands
        .into_iter()
        .find(|command| command.kind == IblBakeComputeKernelKind::IrradianceSh9)
        .expect("PMREM+SH9 request must contain the terminal SH9 command");
    expected.readback_copies.clear();

    let actual = ibl_bake_wgpu_command_plan_for_runtime_kernel(
        &request,
        IblBakeArtifactDescriptor::current_for_runtime_cache_request(&request),
        ibl_bake_irradiance_sh9_kernel_plan(&request),
    );

    assert_eq!(actual, expected);
    assert!(actual.readback_copies.is_empty());
}

#[test]
fn realtime_prefilter_slice_matches_the_selected_full_request_command() {
    let request = request();
    for slice in [
        RealtimeIblPrefilterDispatchSlice {
            mip_level: 0,
            first_face: 0,
            face_count: 2,
        },
        RealtimeIblPrefilterDispatchSlice {
            mip_level: 3,
            first_face: 2,
            face_count: 2,
        },
        RealtimeIblPrefilterDispatchSlice {
            mip_level: 7,
            first_face: 0,
            face_count: 6,
        },
    ] {
        let mut expected = ibl_bake_wgpu_command_plan_for_request(&request)
            .commands
            .into_iter()
            .find(|command| {
                command.kind
                    == IblBakeComputeKernelKind::Pmrem {
                        mip_level: u32::from(slice.mip_level),
                    }
            })
            .expect("default PMREM request must contain the requested mip");
        expected.params.words[5] = u32::from(slice.first_face);
        let writes_all_faces = expected.params.words[7] == 1.0_f32.to_bits()
            && slice.first_face == 0
            && slice.face_count == IBL_BAKE_CUBE_FACE_COUNT as u8;
        if !writes_all_faces {
            expected.params.words[7] = 0;
        }
        expected.dispatch_groups = ibl_bake_pmrem_dispatch_groups_for_face_range(
            request.pmrem_face_size(),
            request.pmrem_mip_count(),
            u32::from(slice.mip_level),
            u32::from(slice.first_face),
            u32::from(slice.face_count),
        )
        .expect("scheduled PMREM slice must have a dispatch extent");
        expected.readback_copies.clear();

        assert_eq!(
            ibl_bake_wgpu_prefilter_command_for_slice(&request, slice),
            Some(expected),
        );
    }
}

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
