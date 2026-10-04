use zircon_runtime::core::framework::ai::{
    AiBehaviorEffectCommand, AiBehaviorEffectId, AiBehaviorEffectOutcome, AiBehaviorEffectReceipt,
    AiBehaviorTreeId, AiBlackboardValue, AiGameplayEvent,
};
use zircon_runtime::core::framework::scene::WorldHandle;
use zircon_runtime::scene::World;

use crate::behavior_tree::RuntimeBehaviorIntegrationHost;
use crate::manager::state::AgentBlackboard;
use crate::DefaultAiManager;

use super::commit_behavior_effects;

#[test]
fn replaying_a_committed_effect_id_suppresses_duplicate_blackboard_and_world_output() {
    let manager = DefaultAiManager::default();
    let world_handle = WorldHandle::new(12);
    let mut world = World::new();
    world.register_event::<AiGameplayEvent>();
    world.register_event::<AiBehaviorEffectReceipt>();
    let command = AiBehaviorEffectCommand::EmitEvent {
        effect_id: AiBehaviorEffectId {
            world: world_handle,
            entity: 17,
            behavior_tree: AiBehaviorTreeId::new(2),
            compiled_tree_generation: 1,
            effect_generation: 3,
            tick: 4,
            tree_id: "replay".to_string(),
            node_id: "emit".to_string(),
            ordinal: 0,
        },
        name: "ai.once".to_string(),
        payload: Some(AiBlackboardValue::Integer(9)),
    };
    let mut blackboard = AgentBlackboard::Dynamic(Vec::new());

    {
        let mut host = RuntimeBehaviorIntegrationHost::new(&mut world, None);
        commit_behavior_effects(
            &manager,
            vec![command.clone()],
            &mut blackboard,
            Some(&mut host),
        )
        .expect("first effect commit");
    }
    world.update_events::<AiGameplayEvent>();
    world.update_events::<AiBehaviorEffectReceipt>();
    assert_eq!(world.events::<AiGameplayEvent>().unwrap().len(), 1);

    {
        let mut host = RuntimeBehaviorIntegrationHost::new(&mut world, None);
        commit_behavior_effects(&manager, vec![command], &mut blackboard, Some(&mut host))
            .expect("duplicate effect is an acknowledged no-op");
    }
    world.update_events::<AiGameplayEvent>();
    world.update_events::<AiBehaviorEffectReceipt>();

    assert!(world.events::<AiGameplayEvent>().unwrap().is_empty());
    let receipt = world
        .events::<AiBehaviorEffectReceipt>()
        .and_then(|events| events.iter().next())
        .expect("duplicate suppression has a typed receipt");
    assert_eq!(
        receipt.outcome,
        AiBehaviorEffectOutcome::DuplicateSuppressed
    );
}
