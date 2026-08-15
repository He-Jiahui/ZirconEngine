use crate::core::framework::render::{
    AntiAliasSettings, PostProcessEffectKind, PostProcessGraphResourceNames,
    PostProcessStackDescriptor, RenderExposureSettings, RenderPostProcessEffectStackSettings,
};
#[test]
fn fxaa_terminal_anti_alias_routes_output_transfer_through_terminal_input() {
    let stack = PostProcessStackDescriptor::from_extract_settings_with_effect_stack_and_anti_alias(
        &Default::default(),
        &Default::default(),
        &RenderPostProcessEffectStackSettings::default(),
        false,
        false,
        &AntiAliasSettings::fxaa(),
    );

    let output_transfer = stack
        .effects
        .iter()
        .find(|effect| effect.kind == PostProcessEffectKind::OutputTransfer)
        .expect("FXAA stack should still transfer postprocess output");
    assert_eq!(
        output_transfer.required_inputs,
        vec![PostProcessGraphResourceNames::FINAL_COMPOSITED.to_string()]
    );
    assert_eq!(
        output_transfer.produced_outputs,
        vec![PostProcessGraphResourceNames::FINAL_COLOR.to_string()]
    );

    let fxaa = stack
        .effects
        .iter()
        .find(|effect| effect.kind == PostProcessEffectKind::Fxaa)
        .expect("FXAA settings should declare the terminal anti-alias node");
    assert!(fxaa.enabled);
    assert_eq!(
        fxaa.required_inputs,
        vec![PostProcessGraphResourceNames::TONEMAPPED.to_string()]
    );
    assert_eq!(
        fxaa.produced_outputs,
        vec![PostProcessGraphResourceNames::FINAL_COMPOSITED.to_string()]
    );
    assert_eq!(fxaa.after, vec![PostProcessEffectKind::Uber]);

    let graph = stack.validated_graph();
    let output_index = graph
        .nodes
        .iter()
        .position(|node| node.kind == PostProcessEffectKind::OutputTransfer)
        .expect("validated graph should keep final transfer");
    let fxaa_index = graph
        .nodes
        .iter()
        .position(|node| node.kind == PostProcessEffectKind::Fxaa)
        .expect("validated graph should keep enabled FXAA");
    assert!(fxaa_index < output_index);
}

#[test]
fn smaa_terminal_anti_alias_routes_output_transfer_through_terminal_input() {
    let stack = PostProcessStackDescriptor::from_extract_settings_with_effect_stack_and_anti_alias(
        &Default::default(),
        &Default::default(),
        &RenderPostProcessEffectStackSettings::default(),
        false,
        false,
        &AntiAliasSettings::smaa(),
    );

    let output_transfer = stack
        .effects
        .iter()
        .find(|effect| effect.kind == PostProcessEffectKind::OutputTransfer)
        .expect("SMAA stack should still transfer postprocess output");
    assert_eq!(
        output_transfer.required_inputs,
        vec![PostProcessGraphResourceNames::FINAL_COMPOSITED.to_string()]
    );
    assert_eq!(
        output_transfer.produced_outputs,
        vec![PostProcessGraphResourceNames::FINAL_COLOR.to_string()]
    );

    let smaa = stack
        .effects
        .iter()
        .find(|effect| effect.kind == PostProcessEffectKind::Smaa)
        .expect("SMAA settings should declare the terminal anti-alias node");
    assert!(smaa.enabled);
    assert_eq!(
        smaa.required_inputs,
        vec![PostProcessGraphResourceNames::TONEMAPPED.to_string()]
    );
    assert_eq!(
        smaa.produced_outputs,
        vec![PostProcessGraphResourceNames::FINAL_COMPOSITED.to_string()]
    );
    assert_eq!(smaa.after, vec![PostProcessEffectKind::Uber]);
    assert!(!stack
        .effects
        .iter()
        .any(|effect| effect.kind == PostProcessEffectKind::Fxaa));

    let graph = stack.validated_graph();
    let output_index = graph
        .nodes
        .iter()
        .position(|node| node.kind == PostProcessEffectKind::OutputTransfer)
        .expect("validated graph should keep final transfer");
    let smaa_index = graph
        .nodes
        .iter()
        .position(|node| node.kind == PostProcessEffectKind::Smaa)
        .expect("validated graph should keep enabled SMAA terminal pass");
    assert!(smaa_index < output_index);
}

#[test]
fn dynamic_resolution_upscales_terminal_anti_alias_before_device_output() {
    let stack = PostProcessStackDescriptor::from_extract_settings_with_effect_stack_exposure_anti_alias_and_upscale(
        &Default::default(),
        &Default::default(),
        RenderExposureSettings::default(),
        &RenderPostProcessEffectStackSettings::default(),
        false,
        false,
        &AntiAliasSettings::fxaa(),
        true,
    );

    let fxaa = stack
        .effects
        .iter()
        .find(|effect| effect.kind == PostProcessEffectKind::Fxaa)
        .expect("FXAA settings should declare the terminal anti-alias node");
    assert_eq!(
        fxaa.produced_outputs,
        vec![PostProcessGraphResourceNames::FINAL_COMPOSITED.to_string()]
    );

    let upscale = stack
        .effects
        .iter()
        .find(|effect| effect.kind == PostProcessEffectKind::Upscale)
        .expect("dynamic resolution should declare an explicit upscale node");
    assert_eq!(
        upscale.required_inputs,
        vec![PostProcessGraphResourceNames::FINAL_COMPOSITED.to_string()]
    );
    assert_eq!(upscale.after, vec![PostProcessEffectKind::Fxaa]);

    let output_transfer = stack
        .effects
        .iter()
        .find(|effect| effect.kind == PostProcessEffectKind::OutputTransfer)
        .expect("dynamic resolution stack should keep final transfer");
    assert_eq!(
        output_transfer.required_inputs,
        vec![PostProcessGraphResourceNames::UPSCALED.to_string()]
    );
    assert_eq!(
        output_transfer.produced_outputs,
        vec![PostProcessGraphResourceNames::FINAL_COLOR.to_string()]
    );

    let graph = stack.validated_graph();
    let fxaa_index = graph
        .nodes
        .iter()
        .position(|node| node.kind == PostProcessEffectKind::Fxaa)
        .expect("validated graph should keep enabled FXAA");
    let upscale_index = graph
        .nodes
        .iter()
        .position(|node| node.kind == PostProcessEffectKind::Upscale)
        .expect("validated graph should keep upscale");
    let output_index = graph
        .nodes
        .iter()
        .position(|node| node.kind == PostProcessEffectKind::OutputTransfer)
        .expect("validated graph should keep final transfer");
    assert!(fxaa_index < upscale_index);
    assert!(upscale_index < output_index);
}

#[test]
fn dynamic_resolution_declares_upscale_before_output_transfer() {
    let stack = PostProcessStackDescriptor::from_extract_settings_with_effect_stack_exposure_anti_alias_and_upscale(
        &Default::default(),
        &Default::default(),
        RenderExposureSettings::default(),
        &RenderPostProcessEffectStackSettings::default(),
        false,
        false,
        &AntiAliasSettings::off(),
        true,
    );

    let upscale = stack
        .effects
        .iter()
        .find(|effect| effect.kind == PostProcessEffectKind::Upscale)
        .expect("dynamic resolution should declare an explicit upscale node");
    assert_eq!(
        upscale.required_inputs,
        vec![PostProcessGraphResourceNames::TONEMAPPED.to_string()]
    );
    assert_eq!(
        upscale.produced_outputs,
        vec![PostProcessGraphResourceNames::UPSCALED.to_string()]
    );

    let output_transfer = stack
        .effects
        .iter()
        .find(|effect| effect.kind == PostProcessEffectKind::OutputTransfer)
        .expect("dynamic resolution stack should keep final transfer");
    assert_eq!(
        output_transfer.required_inputs,
        vec![PostProcessGraphResourceNames::UPSCALED.to_string()]
    );
    assert!(output_transfer
        .after
        .contains(&PostProcessEffectKind::Upscale));

    let graph = stack.validated_graph();
    let uber_index = graph
        .nodes
        .iter()
        .position(|node| node.kind == PostProcessEffectKind::Uber)
        .expect("validated graph should include the tonemap source");
    let upscale_index = graph
        .nodes
        .iter()
        .position(|node| node.kind == PostProcessEffectKind::Upscale)
        .expect("validated graph should keep upscale");
    let output_index = graph
        .nodes
        .iter()
        .position(|node| node.kind == PostProcessEffectKind::OutputTransfer)
        .expect("validated graph should keep final transfer");
    assert!(uber_index < upscale_index);
    assert!(upscale_index < output_index);
}
