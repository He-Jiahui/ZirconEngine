use crate::core::framework::render::{
    AntiAliasSettings, PostProcessEffectKind, PostProcessGraphResourceNames,
    PostProcessStackDescriptor, RenderExposureMode, RenderExposureSettings,
    RenderPostProcessEffectStackSettings,
};
#[test]
fn manual_exposure_declares_resolve_without_histogram() {
    let stack =
        PostProcessStackDescriptor::from_extract_settings_with_effect_stack_exposure_and_anti_alias(
            &Default::default(),
            &Default::default(),
            RenderExposureSettings {
                mode: RenderExposureMode::Manual,
                ..Default::default()
            },
            &RenderPostProcessEffectStackSettings::default(),
            false,
            false,
            &AntiAliasSettings::off(),
        );

    assert!(stack
        .initial_resources
        .contains(&PostProcessGraphResourceNames::EXPOSURE_PREVIOUS.to_string()));
    assert!(!stack
        .effects
        .iter()
        .any(|effect| effect.kind == PostProcessEffectKind::ExposureHistogram));
    let resolve = stack
        .effects
        .iter()
        .find(|effect| effect.kind == PostProcessEffectKind::ExposureResolve)
        .expect("manual exposure still writes the unified exposure buffer");
    assert_eq!(
        resolve.required_inputs,
        vec![PostProcessGraphResourceNames::EXPOSURE_PREVIOUS.to_string()]
    );
    assert_eq!(
        resolve.produced_outputs,
        vec![PostProcessGraphResourceNames::EXPOSURE_CURRENT.to_string()]
    );

    let graph = stack.validated_graph();
    let resolve_index = graph
        .nodes
        .iter()
        .position(|node| node.kind == PostProcessEffectKind::ExposureResolve)
        .expect("validated graph should keep exposure resolve");
    let output_index = graph
        .nodes
        .iter()
        .position(|node| node.kind == PostProcessEffectKind::OutputTransfer)
        .expect("validated graph should keep final transfer");
    assert!(resolve_index < output_index);
}

#[test]
fn default_stack_declares_light_list_for_uber_cluster_bind_group() {
    let stack = PostProcessStackDescriptor::default();

    assert!(stack
        .initial_resources
        .contains(&PostProcessGraphResourceNames::LIGHT_LIST.to_string()));
}

#[test]
fn histogram_exposure_declares_histogram_before_resolve() {
    let stack =
        PostProcessStackDescriptor::from_extract_settings_with_effect_stack_exposure_and_anti_alias(
            &Default::default(),
            &Default::default(),
            RenderExposureSettings {
                mode: RenderExposureMode::Histogram,
                ..Default::default()
            },
            &RenderPostProcessEffectStackSettings::default(),
            false,
            false,
            &AntiAliasSettings::off(),
        );

    let histogram = stack
        .effects
        .iter()
        .find(|effect| effect.kind == PostProcessEffectKind::ExposureHistogram)
        .expect("histogram mode should build the histogram node");
    assert_eq!(
        histogram.required_inputs,
        vec![PostProcessGraphResourceNames::SCENE_COLOR.to_string()]
    );
    assert_eq!(
        histogram.produced_outputs,
        vec![PostProcessGraphResourceNames::EXPOSURE_HISTOGRAM.to_string()]
    );

    let resolve = stack
        .effects
        .iter()
        .find(|effect| effect.kind == PostProcessEffectKind::ExposureResolve)
        .expect("histogram mode should resolve exposure");
    assert!(resolve
        .required_inputs
        .contains(&PostProcessGraphResourceNames::EXPOSURE_HISTOGRAM.to_string()));
    assert!(resolve
        .after
        .contains(&PostProcessEffectKind::ExposureHistogram));

    let graph = stack.validated_graph();
    let histogram_index = graph
        .nodes
        .iter()
        .position(|node| node.kind == PostProcessEffectKind::ExposureHistogram)
        .expect("validated graph should keep exposure histogram");
    let resolve_index = graph
        .nodes
        .iter()
        .position(|node| node.kind == PostProcessEffectKind::ExposureResolve)
        .expect("validated graph should keep exposure resolve");
    let output_index = graph
        .nodes
        .iter()
        .position(|node| node.kind == PostProcessEffectKind::OutputTransfer)
        .expect("validated graph should keep final transfer");
    assert!(histogram_index < resolve_index);
    assert!(resolve_index < output_index);
}
