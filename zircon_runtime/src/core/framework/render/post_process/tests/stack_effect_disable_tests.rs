use super::{PostProcessEffectKind, PostProcessEffectSettings, PostProcessStackDescriptor};

fn effect(
    kind: PostProcessEffectKind,
    required_inputs: &[&str],
    produced_outputs: &[&str],
    after: &[PostProcessEffectKind],
) -> PostProcessEffectSettings {
    PostProcessEffectSettings::new(kind)
        .with_required_inputs(required_inputs.iter().copied())
        .with_produced_outputs(produced_outputs.iter().copied())
        .with_after(after.iter().copied())
}

#[test]
fn effect_disable_preserves_provider_output_metadata() {
    let stack = PostProcessStackDescriptor {
        initial_resources: vec![],
        effects: vec![
            effect(PostProcessEffectKind::Bloom, &[], &["bloom.output"], &[]),
            effect(
                PostProcessEffectKind::Uber,
                &["bloom.output", "scene.color"],
                &["final.color"],
                &[PostProcessEffectKind::Bloom],
            ),
        ],
    };

    let disabled = stack.with_effect_disabled(PostProcessEffectKind::Bloom);

    assert!(!disabled.effects[0].enabled);
    assert_eq!(disabled.effects[0].produced_outputs, ["bloom.output"]);
    assert_eq!(disabled.effects[1].required_inputs, ["scene.color"]);
    assert!(disabled.effects[1].after.is_empty());
}

#[test]
fn effect_disable_indexes_outputs_from_every_matching_provider() {
    let stack = PostProcessStackDescriptor {
        initial_resources: vec![],
        effects: vec![
            effect(PostProcessEffectKind::Bloom, &[], &["bloom.a"], &[]),
            effect(PostProcessEffectKind::Bloom, &[], &["bloom.b"], &[]),
            effect(
                PostProcessEffectKind::Uber,
                &["bloom.a", "scene.color", "bloom.b"],
                &[],
                &[PostProcessEffectKind::Bloom],
            ),
        ],
    };

    let disabled = stack.with_effect_disabled(PostProcessEffectKind::Bloom);

    assert!(disabled.effects[..2].iter().all(|effect| !effect.enabled));
    assert_eq!(disabled.effects[0].produced_outputs, ["bloom.a"]);
    assert_eq!(disabled.effects[1].produced_outputs, ["bloom.b"]);
    assert_eq!(disabled.effects[2].required_inputs, ["scene.color"]);
}

#[test]
fn effect_disable_removes_dangling_dependency_without_a_provider() {
    let stack = PostProcessStackDescriptor {
        initial_resources: vec![],
        effects: vec![effect(
            PostProcessEffectKind::Uber,
            &["scene.color"],
            &["final.color"],
            &[PostProcessEffectKind::Bloom],
        )],
    };

    let disabled = stack.with_effect_disabled(PostProcessEffectKind::Bloom);

    assert!(disabled.effects[0].enabled);
    assert_eq!(disabled.effects[0].required_inputs, ["scene.color"]);
    assert_eq!(disabled.effects[0].produced_outputs, ["final.color"]);
    assert!(disabled.effects[0].after.is_empty());
}
