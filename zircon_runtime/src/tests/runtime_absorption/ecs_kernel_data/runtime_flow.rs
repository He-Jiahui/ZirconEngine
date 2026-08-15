use super::support::assert_source_anchors;

pub(super) fn assert_runtime_08_flow_anchors() {
    let observer_sources = [
        include_str!("../../../scene/ecs/observer/mod.rs"),
        include_str!("../../../scene/ecs/observer/callback_registry.rs"),
        include_str!("../../../scene/ecs/observer/callbacks.rs"),
        include_str!("../../../scene/ecs/observer/entry.rs"),
        include_str!("../../../scene/ecs/observer/id.rs"),
        include_str!("../../../scene/ecs/observer/store.rs"),
        include_str!("../../../scene/world/observers.rs"),
    ];
    assert_source_anchors(
        "Runtime 08 observer",
        &observer_sources,
        &[
            "pub struct ObserverStore",
            "pub fn observe_lifecycle(",
            "pub fn observe_event<E>(",
            "pub fn observe_entity_event<E>(",
            "pub fn remove(&mut self, id: ObserverId) -> SceneResult<()>",
            "pub(crate) fn lifecycle_callbacks(",
            "lifecycle_buckets: HashMap<LifecycleObserverKey, Arc<BTreeMap<ObserverId, LifecycleObserver>>>",
            "observer_locations: HashMap<ObserverId, ObserverBucket>",
        ],
    );
    let observer_source = observer_sources.join("\n");
    for retired_anchor in [
        "pub fn remove(&mut self, id: ObserverId) -> bool",
        "lifecycle_buckets: HashMap<LifecycleObserverKey, Arc<[LifecycleObserver]>>",
        "let mut callbacks = Vec::with_capacity(callback_count);",
        "callbacks.push(observer.callback.clone());",
    ] {
        assert!(
            !observer_source.contains(retired_anchor),
            "Runtime 08 observer hard cut must not retain `{retired_anchor}`"
        );
    }
    assert_source_anchors(
        "Runtime 08 deferred command",
        &[
            include_str!("../../../scene/ecs/commands/command.rs"),
            include_str!("../../../scene/ecs/commands/command_queue.rs"),
            include_str!("../../../scene/ecs/commands/commands/mod.rs"),
            include_str!("../../../scene/ecs/commands/commands/entity_commands.rs"),
            include_str!("../../../scene/ecs/commands/commands/facade.rs"),
            include_str!("../../../scene/ecs/commands/commands/param.rs"),
            include_str!("../../../scene/world/commands.rs"),
        ],
        &[
            "pub enum DeferredCommandOperation",
            "pub struct DeferredCommandError",
            "pub struct DeferredCommandReport",
            "pub fn errors(&self) -> &[DeferredCommandError]",
            "pub fn apply(&mut self, world: &mut World) -> DeferredCommandReport",
            "world.record_deferred_command_error(DeferredCommandError::new(",
            "DeferredCommandOperation::Despawn",
            "DeferredCommandOperation::Insert",
            "DeferredCommandOperation::Remove",
            "pub fn apply_deferred(&mut self) -> DeferredCommandReport",
            "std::mem::take(&mut self.deferred_command_errors)",
        ],
    );
    assert_source_anchors(
        "Runtime 08 event/message",
        &[
            include_str!("../../../scene/ecs/events/mod.rs"),
            include_str!("../../../scene/ecs/events/cursor.rs"),
            include_str!("../../../scene/ecs/events/id.rs"),
            include_str!("../../../scene/ecs/events/metrics.rs"),
            include_str!("../../../scene/ecs/events/queue.rs"),
            include_str!("../../../scene/ecs/events/store.rs"),
            include_str!("../../../scene/ecs/events/subscription.rs"),
            include_str!("../../../scene/ecs/messages/mod.rs"),
            include_str!("../../../scene/ecs/messages/cursor.rs"),
            include_str!("../../../scene/ecs/messages/id.rs"),
            include_str!("../../../scene/ecs/messages/queue.rs"),
            include_str!("../../../scene/ecs/messages/store.rs"),
            include_str!("../../../scene/world/events.rs"),
            include_str!("../../../scene/world/messages.rs"),
        ],
        &[
            "pub struct Events<T>",
            "current: Vec<T>",
            "next: Vec<T>",
            "pub fn update(&mut self)",
            "std::mem::swap(&mut self.current, &mut self.next);",
            "self.current.clear();",
            "self.next.clear();",
            "pub fn update_all(&mut self)",
            "pub struct MessageId<T>",
            "pub struct Messages<T>",
            "next_id: usize",
            "pub fn clear(&mut self)",
        ],
    );
}
