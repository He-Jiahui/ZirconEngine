use std::collections::BTreeMap;

use zircon_runtime::core::framework::animation::AnimationParameterValue;
use zircon_runtime::core::resource::{AnimationStateMachineMarker, ResourceHandle, ResourceId};
use zircon_runtime::scene::components::AnimationStateMachinePlayerComponent;

use super::apply_active_state_update;

#[test]
fn one_shot_trigger_commit_consumes_only_current_trigger_values() {
    let mut player = AnimationStateMachinePlayerComponent {
        state_machine: ResourceHandle::<AnimationStateMachineMarker>::new(
            ResourceId::from_stable_label("one-shot trigger commit"),
        ),
        parameters: BTreeMap::from([
            ("fire".into(), AnimationParameterValue::Trigger),
            ("jump".into(), AnimationParameterValue::Trigger),
            ("speed".into(), AnimationParameterValue::Scalar(2.0)),
            ("grounded".into(), AnimationParameterValue::Bool(true)),
        ])
        .into(),
        active_state: Some("Idle".into()),
        playing: true,
    };

    apply_active_state_update(
        &mut player,
        Some("Run".into()),
        &["fire".into(), "speed".into()],
    );

    assert_eq!(player.active_state.as_deref(), Some("Run"));
    assert!(!player.parameters.contains_key("fire"));
    assert_eq!(
        player.parameters.get("jump"),
        Some(&AnimationParameterValue::Trigger)
    );
    assert_eq!(
        player.parameters.get("speed"),
        Some(&AnimationParameterValue::Scalar(2.0))
    );
    assert_eq!(
        player.parameters.get("grounded"),
        Some(&AnimationParameterValue::Bool(true))
    );
}
