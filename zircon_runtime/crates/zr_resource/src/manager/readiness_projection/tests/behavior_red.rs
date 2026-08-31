use super::*;

#[test]
#[ignore = "RED: readiness cycles must fail closed before the production graph hard cut"]
fn self_and_multi_node_cycles_do_not_publish_recursive_loaded() {
    let self_record = ready_record("cycle/self", Vec::new());
    let self_id = self_record.id;
    let self_record = self_record.with_dependency_ids(vec![self_id]);
    let mut self_projection = ResourceReadinessProjection::default();
    self_projection.apply_updates([source_update(self_record)]);

    assert_eq!(
        self_projection
            .generation
            .row(self_id)
            .expect("self-cycle readiness row")
            .recursive_dependency_state,
        ResourceReadinessState::Failed
    );

    let first = ready_record("cycle/first", Vec::new());
    let second = ready_record("cycle/second", vec![first.id]);
    let first = first.with_dependency_ids(vec![second.id]);
    let ids = [first.id, second.id];
    let mut projection = ResourceReadinessProjection::default();
    projection.apply_updates([source_update(first), source_update(second)]);

    for id in ids {
        assert_eq!(
            projection
                .generation
                .row(id)
                .expect("cycle readiness row")
                .recursive_dependency_state,
            ResourceReadinessState::Failed,
            "cycle member {id} advertised recursive readiness"
        );
    }
}

#[test]
fn cycle_failure_propagates_to_dependants_independent_of_update_order() {
    let first = ready_record("cycle-order/first", Vec::new());
    let second = ready_record("cycle-order/second", vec![first.id]);
    let first = first.with_dependency_ids(vec![second.id]);
    let dependant = ready_record("cycle-order/dependant", vec![first.id]);
    let expected_failed = [first.id, second.id, dependant.id];

    for updates in [
        vec![
            source_update(first.clone()),
            source_update(second.clone()),
            source_update(dependant.clone()),
        ],
        vec![
            source_update(dependant.clone()),
            source_update(second.clone()),
            source_update(first.clone()),
        ],
    ] {
        let mut projection = ResourceReadinessProjection::default();
        projection.apply_updates(updates);
        for id in expected_failed {
            assert_eq!(
                projection
                    .generation
                    .row(id)
                    .expect("cycle or dependant readiness row")
                    .recursive_dependency_state,
                ResourceReadinessState::Failed,
                "cycle failure did not propagate to {id}"
            );
        }
    }
}

#[test]
fn removing_a_cycle_member_recomputes_and_recovers_the_remaining_graph() {
    let first = ready_record("cycle-removal/first", Vec::new());
    let second = ready_record("cycle-removal/second", vec![first.id]);
    let first = first.with_dependency_ids(vec![second.id]);
    let first_id = first.id;
    let second_id = second.id;
    let mut projection = ResourceReadinessProjection::default();
    projection.apply_updates([source_update(first), source_update(second)]);

    projection.apply_updates([ResourceReadinessSourceUpdate {
        id: first_id,
        record: None,
        runtime_state: RuntimeResourceState::Unloaded,
        payload_type_id: None,
    }]);

    assert!(projection.generation.row(first_id).is_none());
    assert_eq!(
        projection
            .generation
            .row(second_id)
            .expect("remaining cycle member readiness row")
            .recursive_dependency_state,
        ResourceReadinessState::Failed,
        "the remaining node still references the removed dependency and must fail closed"
    );

    let detached = ready_record("cycle-removal/second", Vec::new());
    projection.apply_updates([source_update(detached)]);
    assert_eq!(
        projection
            .generation
            .row(second_id)
            .expect("detached readiness row")
            .recursive_dependency_state,
        ResourceReadinessState::Loaded
    );
}

#[test]
fn duplicate_and_reordered_dependency_sets_preserve_generation_identity() {
    let first_dependency = ready_record("canonical/first", Vec::new());
    let second_dependency = ready_record("canonical/second", Vec::new());
    let first_dependency_id = first_dependency.id;
    let second_dependency_id = second_dependency.id;
    let parent = ready_record(
        "canonical/parent",
        vec![first_dependency_id, second_dependency_id],
    );
    let mut projection = ResourceReadinessProjection::default();
    projection.apply_updates([
        source_update(first_dependency),
        source_update(second_dependency),
        source_update(parent.clone()),
    ]);
    let canonical_generation = projection.generation();

    projection.apply_updates([source_update(parent.clone().with_dependency_ids(vec![
        parent.dependency_ids[1],
        parent.dependency_ids[0],
        parent.dependency_ids[1],
    ]))]);
    assert!(
        Arc::ptr_eq(&canonical_generation, &projection.generation()),
        "dependency order changed the published generation"
    );

    projection.apply_updates([source_update(parent.with_dependency_ids(vec![
        first_dependency_id,
        second_dependency_id,
        first_dependency_id,
    ]))]);
    assert!(
        Arc::ptr_eq(&canonical_generation, &projection.generation()),
        "duplicate dependency IDs changed the published generation"
    );
}

#[test]
fn dependency_arrival_replacement_and_removal_update_the_exact_reverse_closure() {
    let dependency = ready_record("edges/dependency", Vec::new());
    let replacement = ready_record("edges/replacement", Vec::new());
    let parent = ready_record("edges/parent", vec![dependency.id]);
    let parent_id = parent.id;
    let mut projection = ResourceReadinessProjection::default();

    projection.apply_updates([source_update(parent.clone())]);
    assert_eq!(
        projection
            .generation
            .row(parent_id)
            .expect("parent with missing dependency")
            .recursive_dependency_state,
        ResourceReadinessState::Failed
    );

    projection.apply_updates([source_update(dependency)]);
    assert_eq!(
        projection
            .generation
            .row(parent_id)
            .expect("parent after dependency arrival")
            .recursive_dependency_state,
        ResourceReadinessState::Loaded
    );

    projection.apply_updates([source_update(
        parent.clone().with_dependency_ids(vec![replacement.id]),
    )]);
    assert_eq!(
        projection
            .generation
            .row(parent_id)
            .expect("parent after dependency replacement")
            .recursive_dependency_state,
        ResourceReadinessState::Failed
    );

    projection.apply_updates([source_update(parent.with_dependency_ids(Vec::new()))]);
    let detached_parent = projection
        .generation
        .row(parent_id)
        .expect("parent after dependency removal")
        .clone();
    assert_eq!(
        detached_parent.recursive_dependency_state,
        ResourceReadinessState::Loaded
    );

    projection.apply_updates([source_update(replacement)]);
    assert!(
        Arc::ptr_eq(
            &detached_parent,
            projection
                .generation
                .row(parent_id)
                .expect("detached parent after old dependency arrival")
        ),
        "a detached dependency still invalidated its former parent"
    );
}

#[test]
#[ignore = "isolated RED: proves the current recursive evaluator consumes graph-depth call stack"]
fn deep_chain_10000_publishes_without_native_stack_growth() {
    const NODE_COUNT: usize = 10_000;
    let ids = (0..NODE_COUNT)
        .map(|index| ResourceId::from_stable_label(&format!("readiness-deep-chain-{index}")))
        .collect::<Vec<_>>();
    let records = ids
        .iter()
        .enumerate()
        .map(|(index, id)| {
            let locator =
                ResourceLocator::parse(&format!("res://readiness/deep-chain/{index:05}.asset"))
                    .expect("valid deep-chain locator");
            ResourceRecord::new(*id, ResourceKind::Data, locator)
                .with_state(ResourceState::Ready)
                .with_dependency_ids(ids.get(index + 1).copied().into_iter().collect())
        })
        .map(source_update)
        .collect::<Vec<_>>();
    let mut projection = ResourceReadinessProjection::default();

    projection.apply_updates(records);

    assert_eq!(projection.generation.diagnostics().row_count, NODE_COUNT);
    assert_eq!(
        projection
            .generation
            .row(ids[0])
            .expect("deep-chain root")
            .recursive_dependency_state,
        ResourceReadinessState::Loaded
    );
}
