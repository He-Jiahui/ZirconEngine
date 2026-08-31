use std::hint::black_box;
use std::time::Instant;

use super::*;

const TARGET_P95_PERCENT: u128 = 70;

#[test]
fn optimization_batch_20260828hx_editor_custom_payload_keeps_owned_allocations() {
    let call = benchmark_call(8, 4 * 1024);
    let symbol_allocation = call.symbol.as_ptr();
    let arguments_allocation = call.arguments.as_ptr();

    let payload = EditorUiBindingPayload::from_call(call).expect("decode custom payload");

    let EditorUiBindingPayload::Custom(call) = payload else {
        panic!("expected custom binding payload");
    };
    assert_eq!(call.symbol.as_ptr(), symbol_allocation);
    assert_eq!(call.arguments.as_ptr(), arguments_allocation);
}

#[test]
fn optimization_batch_20260828hx_editor_routes_one_family_and_preserves_payload_errors() {
    assert_eq!(
        command_family("AnimationCommand.AddKey"),
        Some(CommandFamily::Animation)
    );
    assert_eq!(
        command_family("SelectionCommand.SelectSceneNode"),
        Some(CommandFamily::Selection)
    );
    assert_eq!(
        command_family("AssetCommand.OpenAsset"),
        Some(CommandFamily::Asset)
    );
    assert_eq!(
        command_family("WelcomeCommand.CreateProject"),
        Some(CommandFamily::Welcome)
    );
    assert_eq!(
        command_family("DraftCommand.SetInspectorField"),
        Some(CommandFamily::Draft)
    );
    assert_eq!(
        command_family("DockCommand.FocusView"),
        Some(CommandFamily::Dock)
    );
    assert_eq!(
        command_family("ViewportCommand.PointerMoved"),
        Some(CommandFamily::Viewport)
    );
    assert_eq!(command_family("Plugin.CustomPayload"), None);
    assert_eq!(command_family("MenuAction"), None);

    let malformed = UiBindingCall::new("ViewportCommand.PointerMoved");
    assert!(EditorUiBindingPayload::from_call(malformed).is_err());
}

#[test]
#[ignore = "release performance contract; run through the managed validation coordinator"]
fn optimization_batch_20260828hx_editor_symbol_routed_binding_decode_benchmark() {
    const SAMPLES: usize = 11;
    const ITERATIONS: usize = 32;
    let call = benchmark_call(256, 1024);

    black_box(legacy_decode(call.clone()).unwrap());
    black_box(EditorUiBindingPayload::from_call(call.clone()).unwrap());

    let mut legacy_samples = Vec::with_capacity(SAMPLES);
    let mut optimized_samples = Vec::with_capacity(SAMPLES);
    for sample_index in 0..SAMPLES {
        let legacy_inputs = (0..ITERATIONS).map(|_| call.clone()).collect::<Vec<_>>();
        let optimized_inputs = (0..ITERATIONS).map(|_| call.clone()).collect::<Vec<_>>();
        if sample_index % 2 == 0 {
            legacy_samples.push(measure_decode(legacy_inputs, legacy_decode));
            optimized_samples.push(measure_decode(
                optimized_inputs,
                EditorUiBindingPayload::from_call,
            ));
        } else {
            optimized_samples.push(measure_decode(
                optimized_inputs,
                EditorUiBindingPayload::from_call,
            ));
            legacy_samples.push(measure_decode(legacy_inputs, legacy_decode));
        }
    }

    let legacy_p95_ns = nearest_rank_percentile(&legacy_samples, 95);
    let optimized_p95_ns = nearest_rank_percentile(&optimized_samples, 95);
    println!(
        "EDITOR216_SYMBOL_ROUTED_BINDING_DECODE_BENCH_V1 legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_samples_ns={} optimized_samples_ns={}",
        join_samples(&legacy_samples),
        join_samples(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100)
            <= legacy_p95_ns.saturating_mul(TARGET_P95_PERCENT),
        "optimized P95 {optimized_p95_ns}ns must be at most {TARGET_P95_PERCENT}% of legacy P95 {legacy_p95_ns}ns"
    );
}

fn benchmark_call(argument_count: usize, argument_bytes: usize) -> UiBindingCall {
    let payload = "x".repeat(argument_bytes);
    let mut call = UiBindingCall::new("Plugin.LargeCustomPayload");
    call.arguments = (0..argument_count)
        .map(|index| UiBindingValue::string(format!("argument-{index}-{payload}")))
        .collect();
    call
}

fn legacy_decode(call: UiBindingCall) -> Result<EditorUiBindingPayload, EditorUiBindingError> {
    if let Some(command) = AnimationCommand::from_call(call.clone())? {
        return Ok(EditorUiBindingPayload::AnimationCommand(command));
    }
    if let Some(command) = SelectionCommand::from_call(call.clone())? {
        return Ok(EditorUiBindingPayload::SelectionCommand(command));
    }
    if let Some(command) = AssetCommand::from_call(call.clone())? {
        return Ok(EditorUiBindingPayload::AssetCommand(command));
    }
    if let Some(command) = WelcomeCommand::from_call(call.clone())? {
        return Ok(EditorUiBindingPayload::WelcomeCommand(command));
    }
    if let Some(command) = DraftCommand::from_call(call.clone())? {
        return Ok(EditorUiBindingPayload::DraftCommand(command));
    }
    if let Some(command) = DockCommand::from_call(call.clone())? {
        return Ok(EditorUiBindingPayload::DockCommand(command));
    }
    if let Some(command) = ViewportCommand::from_call(call.clone())? {
        return Ok(EditorUiBindingPayload::ViewportCommand(command));
    }
    Ok(EditorUiBindingPayload::Custom(call))
}

fn measure_decode(
    inputs: Vec<UiBindingCall>,
    mut decode: impl FnMut(UiBindingCall) -> Result<EditorUiBindingPayload, EditorUiBindingError>,
) -> u128 {
    let started = Instant::now();
    for call in inputs {
        black_box(decode(black_box(call)).unwrap());
    }
    started.elapsed().as_nanos()
}

fn nearest_rank_percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    ordered[(ordered.len() * percentile).div_ceil(100) - 1]
}

fn join_samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
