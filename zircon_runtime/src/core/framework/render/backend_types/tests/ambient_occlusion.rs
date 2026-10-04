use super::RenderAmbientOcclusionExecutionFailureFlags as FailureFlags;

#[test]
fn ambient_occlusion_execution_failure_flags_are_a_dense_22_bit_contract() {
    let flags = [
        FailureFlags::MISSING_COMPILED_CONTRACT,
        FailureFlags::GENERATION_MISMATCH,
        FailureFlags::OUTPUT_PRODUCER_MISMATCH,
        FailureFlags::UNEXPECTED_DISABLED_WORK,
        FailureFlags::EVALUATE_PASS,
        FailureFlags::EVALUATE_DISPATCH,
        FailureFlags::EVALUATE_RAW_WRITE,
        FailureFlags::SPATIAL_PASS,
        FailureFlags::SPATIAL_DISPATCH,
        FailureFlags::SPATIAL_RAW_READ,
        FailureFlags::SPATIAL_FINAL_WRITE,
        FailureFlags::LIGHTING_PASS,
        FailureFlags::LIGHTING_FINAL_READ,
        FailureFlags::EVALUATE_PIPELINE_RESOLUTION,
        FailureFlags::SPATIAL_PIPELINE_RESOLUTION,
        FailureFlags::PIPELINE_DEVICE_EPOCH_MISMATCH,
        FailureFlags::UPSAMPLE_PASS,
        FailureFlags::UPSAMPLE_DISPATCH,
        FailureFlags::SPATIAL_INTERMEDIATE_WRITE,
        FailureFlags::UPSAMPLE_SPATIAL_READ,
        FailureFlags::UPSAMPLE_FINAL_WRITE,
        FailureFlags::UPSAMPLE_PIPELINE_RESOLUTION,
    ];

    assert_eq!(flags.len(), 22);
    assert_eq!(
        flags
            .into_iter()
            .fold(0_u32, |bits, flag| bits | flag.bits()),
        (1_u32 << 22) - 1
    );
}
