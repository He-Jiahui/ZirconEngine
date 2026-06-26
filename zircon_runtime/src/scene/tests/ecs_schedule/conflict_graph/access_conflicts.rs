use super::*;

#[test]
fn schedule_conflict_graph_reports_component_write_conflicts_in_same_stage() {
    let mut world = World::empty();
    world.spawn((ScheduleHealth(1), SchedulePlayer)).unwrap();
    let read_health = SystemState::<QueryState<&'static ScheduleHealth>>::new(&mut world).unwrap();
    let write_health =
        SystemState::<QueryState<&'static mut ScheduleHealth>>::new(&mut world).unwrap();
    let health_component = world.component_id::<ScheduleHealth>();

    let graph = ScheduleConflictGraph::from_nodes([
        ScheduleConflictNode::new(
            "read.health",
            SystemStage::Update,
            read_health.access().clone(),
        ),
        ScheduleConflictNode::new(
            "write.health",
            SystemStage::Update,
            write_health.access().clone(),
        ),
    ]);

    assert_eq!(graph.nodes().len(), 2);
    assert!(graph.has_conflicts());
    let edge = &graph.edges()[0];
    assert_eq!(edge.left_system_id(), "read.health");
    assert_eq!(edge.right_system_id(), "write.health");
    assert_eq!(edge.stage(), SystemStage::Update);
    assert_eq!(
        edge.conflicts(),
        &[SystemParamConflictKind::Component(health_component)]
    );
    assert_eq!(graph.conflicts_for("read.health").count(), 1);
}

#[test]
fn schedule_conflict_graph_respects_disjoint_query_filters() {
    let mut world = World::empty();
    type PlayerHealth = QueryState<&'static mut ScheduleHealth, With<SchedulePlayer>>;
    type NonPlayerHealth = QueryState<&'static mut ScheduleHealth, Without<SchedulePlayer>>;
    let player_health = SystemState::<PlayerHealth>::new(&mut world).unwrap();
    let non_player_health = SystemState::<NonPlayerHealth>::new(&mut world).unwrap();

    let graph = ScheduleConflictGraph::from_nodes([
        ScheduleConflictNode::new(
            "write.player-health",
            SystemStage::Update,
            player_health.access().clone(),
        ),
        ScheduleConflictNode::new(
            "write.non-player-health",
            SystemStage::Update,
            non_player_health.access().clone(),
        ),
    ]);

    assert!(!graph.has_conflicts());
    assert!(graph.edges().is_empty());
}

#[test]
fn schedule_conflict_graph_keeps_different_stages_independent() {
    let mut world = World::empty();
    let read_health = SystemState::<QueryState<&'static ScheduleHealth>>::new(&mut world).unwrap();
    let write_health =
        SystemState::<QueryState<&'static mut ScheduleHealth>>::new(&mut world).unwrap();

    let graph = ScheduleConflictGraph::from_nodes([
        ScheduleConflictNode::new(
            "read.health",
            SystemStage::PreUpdate,
            read_health.access().clone(),
        ),
        ScheduleConflictNode::new(
            "write.health",
            SystemStage::PostUpdate,
            write_health.access().clone(),
        ),
    ]);

    assert!(!graph.has_conflicts());
}

#[test]
fn schedule_conflict_graph_reports_resource_write_conflicts() {
    let mut world = World::empty();
    world.insert_resource(ScheduleFrameCounter(0));
    let read_counter = SystemState::<ResParam<ScheduleFrameCounter>>::new(&mut world).unwrap();
    let write_counter = SystemState::<ResMutParam<ScheduleFrameCounter>>::new(&mut world).unwrap();
    let counter_resource = world.resource_id::<ScheduleFrameCounter>();

    let graph = ScheduleConflictGraph::from_nodes([
        ScheduleConflictNode::new(
            "read.frame-counter",
            SystemStage::Update,
            read_counter.access().clone(),
        ),
        ScheduleConflictNode::new(
            "write.frame-counter",
            SystemStage::Update,
            write_counter.access().clone(),
        ),
    ]);

    assert!(read_counter.access().conflicts_with(write_counter.access()));
    let edge = &graph.edges()[0];
    assert_eq!(
        edge.conflicts(),
        &[SystemParamConflictKind::Resource(counter_resource)]
    );
}

#[test]
fn schedule_conflict_graph_reports_event_and_message_write_conflicts() {
    let mut event_reader = SystemParamAccess::default();
    event_reader.add_event_read::<ScheduleHitEvent>().unwrap();
    let mut event_writer = SystemParamAccess::default();
    event_writer.add_event_write::<ScheduleHitEvent>().unwrap();
    let mut message_reader = SystemParamAccess::default();
    message_reader
        .add_message_read::<ScheduleNoticeMessage>()
        .unwrap();
    let mut message_writer = SystemParamAccess::default();
    message_writer
        .add_message_write::<ScheduleNoticeMessage>()
        .unwrap();

    let graph = ScheduleConflictGraph::from_nodes([
        ScheduleConflictNode::new("read.event", SystemStage::Update, event_reader),
        ScheduleConflictNode::new("write.event", SystemStage::Update, event_writer),
        ScheduleConflictNode::new("read.message", SystemStage::Update, message_reader),
        ScheduleConflictNode::new("write.message", SystemStage::Update, message_writer),
    ]);
    let event_type = TypeId::of::<ScheduleHitEvent>();
    let message_type = TypeId::of::<ScheduleNoticeMessage>();

    assert_eq!(graph.edges().len(), 2);
    assert!(graph.edges().iter().any(|edge| {
        edge.conflicts()
            .contains(&SystemParamConflictKind::Event(event_type))
    }));
    assert!(graph.edges().iter().any(|edge| {
        edge.conflicts()
            .contains(&SystemParamConflictKind::Message(message_type))
    }));
}

#[test]
fn schedule_conflict_graph_reports_event_and_message_writer_conflicts() {
    let mut first_event_writer = SystemParamAccess::default();
    first_event_writer
        .add_event_write::<ScheduleHitEvent>()
        .unwrap();
    let mut second_event_writer = SystemParamAccess::default();
    second_event_writer
        .add_event_write::<ScheduleHitEvent>()
        .unwrap();
    let mut first_message_writer = SystemParamAccess::default();
    first_message_writer
        .add_message_write::<ScheduleNoticeMessage>()
        .unwrap();
    let mut second_message_writer = SystemParamAccess::default();
    second_message_writer
        .add_message_write::<ScheduleNoticeMessage>()
        .unwrap();

    let graph = ScheduleConflictGraph::from_nodes([
        ScheduleConflictNode::new("write.event.first", SystemStage::Update, first_event_writer),
        ScheduleConflictNode::new(
            "write.event.second",
            SystemStage::Update,
            second_event_writer,
        ),
        ScheduleConflictNode::new(
            "write.message.first",
            SystemStage::Update,
            first_message_writer,
        ),
        ScheduleConflictNode::new(
            "write.message.second",
            SystemStage::Update,
            second_message_writer,
        ),
    ]);
    let event_type = TypeId::of::<ScheduleHitEvent>();
    let message_type = TypeId::of::<ScheduleNoticeMessage>();

    assert_eq!(graph.edges().len(), 2);
    assert!(graph.edges().iter().any(|edge| {
        edge.conflicts()
            .contains(&SystemParamConflictKind::Event(event_type))
    }));
    assert!(graph.edges().iter().any(|edge| {
        edge.conflicts()
            .contains(&SystemParamConflictKind::Message(message_type))
    }));
}

#[test]
fn schedule_conflict_graph_reports_conservative_world_access_conflicts() {
    let mut world_access = SystemParamAccess::default();
    world_access.add_conservative_world_access();
    let read_only = SystemParamAccess::default();

    let graph = ScheduleConflictGraph::from_nodes([
        ScheduleConflictNode::new("runtime.context", SystemStage::Update, world_access),
        ScheduleConflictNode::new("native.read-only", SystemStage::Update, read_only),
    ]);

    assert!(graph.systems_conflict("runtime.context", "native.read-only"));
    assert_eq!(
        graph.edges()[0].conflicts(),
        &[SystemParamConflictKind::World]
    );
}
