use super::*;
use crate::scene::components::MeshRendererPrimitiveBinding;

const REPLAY_LIMIT: usize = 256;

fn stable_keys(projector: &RenderSceneComponentProjector) -> Vec<u64> {
    let mut keys = projector
        .read()
        .iter()
        .map(|(_, primitive)| primitive.stable_instance_key())
        .collect::<Vec<_>>();
    keys.sort_unstable();
    keys
}

fn reset_selection_counters(projector: &RenderSceneComponentProjector) {
    projector.take_geometry_selection_visits();
    projector.take_geometry_selection_peak_keys();
}

// 目标命中加一次前瞻共同计入访问预算；峰值 key 数量也须受限，避免重复依赖扩大续接队列。
fn assert_selection_work_bounded(
    projector: &RenderSceneComponentProjector,
    resource_count: usize,
    selected_count: usize,
) -> usize {
    let visits = projector.take_geometry_selection_visits();
    assert!(
        visits >= selected_count,
        "every returned key must be visited"
    );
    assert!(
        visits <= resource_count * (REPLAY_LIMIT + 1),
        "duplicate memberships and lookahead must stay inside the work bound: {visits}"
    );
    let peak_keys = projector.take_geometry_selection_peak_keys();
    assert!(
        peak_keys >= selected_count,
        "returned keys must be retained"
    );
    assert!(
        peak_keys <= resource_count + REPLAY_LIMIT + 1,
        "retained heads, targets and lookahead must stay bounded: {peak_keys}"
    );
    visits
}

#[test]
fn render_scene_resource_geometry_replay_bounded_targets_visit_only_shared_model_chunk() {
    const DEPENDENT_COUNT: usize = 10_000;
    let renderer = MeshRenderer::default();
    let model = UntypedResourceHandle::new(renderer.model.id(), ResourceKind::Model);
    let (mut projector, _) =
        projector_with_renderers((0..DEPENDENT_COUNT).map(|_| renderer.clone()));
    let expected = stable_keys(&projector);
    let resources = [(model, 1)];
    let mut offset = 0;

    while offset < DEPENDENT_COUNT {
        reset_selection_counters(&projector);
        let selection =
            projector.prepare_geometry_replay_with_revisions(&resources, false, REPLAY_LIMIT, None);
        let end = (offset + REPLAY_LIMIT).min(DEPENDENT_COUNT);
        assert_eq!(selection.target_keys(), &expected[offset..end]);
        assert_eq!(selection.truncated(), end < DEPENDENT_COUNT);
        assert_eq!(
            assert_selection_work_bounded(&projector, 1, selection.selected_primitive_count()),
            selection.selected_primitive_count() + usize::from(selection.truncated()),
            "a shared model must read only this chunk and one lookahead"
        );
        projector.finish_geometry_replay_selection(&selection);
        offset = end;
    }
    assert_eq!(offset, DEPENDENT_COUNT);
}

#[test]
fn render_scene_resource_geometry_replay_bounded_targets_deduplicate_overlapping_resources() {
    const DEPENDENT_COUNT: usize = REPLAY_LIMIT * 2;
    let base = MeshRenderer::default();
    let model = UntypedResourceHandle::new(base.model.id(), ResourceKind::Model);
    let mesh_a = ResourceHandle::<MeshMarker>::new(ResourceId::from_stable_label("bounded/mesh-a"));
    let mesh_b = ResourceHandle::<MeshMarker>::new(ResourceId::from_stable_label("bounded/mesh-b"));
    let resource_a = UntypedResourceHandle::new(mesh_a.id(), ResourceKind::Mesh);
    let resource_b = UntypedResourceHandle::new(mesh_b.id(), ResourceKind::Mesh);
    let (mut projector, _) = projector_with_renderers((0..DEPENDENT_COUNT).map(|index| {
        let mut renderer = base.clone();
        for mesh in [mesh_a, mesh_b] {
            if index % 3 == 2 || (index % 3 == 0) == (mesh == mesh_a) {
                renderer.primitives.push(MeshRendererPrimitiveBinding {
                    mesh,
                    material: renderer.material,
                });
            }
        }
        renderer
    }));
    let expected = stable_keys(&projector);
    let resources = [(resource_b, 2), (model, 2), (resource_a, 2), (model, 2)];

    reset_selection_counters(&projector);
    let first =
        projector.prepare_geometry_replay_with_revisions(&resources, false, REPLAY_LIMIT, None);
    assert_eq!(first.target_keys(), &expected[..REPLAY_LIMIT]);
    assert!(first.truncated());
    let visits = assert_selection_work_bounded(&projector, 3, first.selected_primitive_count());
    assert!(visits >= first.selected_primitive_count() * 2);
    projector.finish_geometry_replay_selection(&first);

    // Resource order and repeated identities do not change the exact continuation scope.
    let reordered = [
        (resource_a, 2),
        (resource_b, 2),
        (model, 2),
        (resource_b, 2),
    ];
    reset_selection_counters(&projector);
    let second =
        projector.prepare_geometry_replay_with_revisions(&reordered, false, REPLAY_LIMIT, None);
    assert_eq!(second.target_keys(), &expected[REPLAY_LIMIT..]);
    assert!(
        !second.truncated(),
        "duplicate keys must not imply another chunk"
    );
    let visits = assert_selection_work_bounded(&projector, 3, second.selected_primitive_count());
    assert!(visits >= second.selected_primitive_count() * 2);
}

#[test]
fn render_scene_resource_geometry_replay_bounded_targets_keep_failed_retry_atomic() {
    let renderer = MeshRenderer::default();
    let model = UntypedResourceHandle::new(renderer.model.id(), ResourceKind::Model);
    let (mut projector, _) = projector_with_renderers((0..300).map(|_| renderer.clone()));
    let first = projector.prepare_geometry_replay(&[model], false, REPLAY_LIMIT, None);
    let initial_generation = projector.read().generation();
    let mut failing = TestResolver::new(2, 2.0);
    failing.issue = Some(RenderSceneGeometryResolveIssue::Pending);
    assert!(projector
        .replay_resource_geometry_with_staging(&[model], false, REPLAY_LIMIT, &mut failing, |_| {
            Ok::<(), std::convert::Infallible>(())
        })
        .is_err());
    assert!(projector
        .replay_resource_geometry_with_staging(
            &[model],
            false,
            REPLAY_LIMIT,
            &mut TestResolver::new(2, 2.0),
            |_| Err::<(), _>("residency rejected"),
        )
        .is_err());
    assert_eq!(projector.read().generation(), initial_generation);
    assert!(projector
        .read()
        .iter()
        .all(|(_, primitive)| primitive.local_bounds().max == [1.0; 3]));

    reset_selection_counters(&projector);
    let retry = projector.prepare_geometry_replay(&[model], false, REPLAY_LIMIT, None);
    assert_eq!(retry.target_keys(), first.target_keys());
    assert!(retry.truncated());
    assert_eq!(
        assert_selection_work_bounded(&projector, 1, retry.selected_primitive_count()),
        REPLAY_LIMIT + 1
    );
    let mut resolver = TestResolver::new(2, 2.0);
    projector
        .replay_resource_geometry_with_staging(&[model], false, REPLAY_LIMIT, &mut resolver, |_| {
            Ok::<(), std::convert::Infallible>(())
        })
        .unwrap();
    projector.finish_geometry_replay_selection(&retry);
    assert_eq!(resolver.calls, REPLAY_LIMIT);
    assert_eq!(
        projector
            .read()
            .iter()
            .filter(|(_, primitive)| primitive.local_bounds().max == [2.0; 3])
            .count(),
        REPLAY_LIMIT
    );
    reset_selection_counters(&projector);
    let remaining = projector.prepare_geometry_replay(&[model], false, REPLAY_LIMIT, None);
    assert_eq!(remaining.selected_primitive_count(), 44);
    assert!(!remaining.truncated());
    assert_eq!(
        assert_selection_work_bounded(&projector, 1, remaining.selected_primitive_count()),
        44
    );
}

#[test]
fn render_scene_resource_geometry_replay_bounded_targets_restart_changed_revision() {
    let renderer = MeshRenderer::default();
    let model = UntypedResourceHandle::new(renderer.model.id(), ResourceKind::Model);
    let (mut projector, _) = projector_with_renderers((0..300).map(|_| renderer.clone()));
    let first =
        projector.prepare_geometry_replay_with_revisions(&[(model, 1)], false, REPLAY_LIMIT, None);
    projector.finish_geometry_replay_selection(&first);
    reset_selection_counters(&projector);
    let continued =
        projector.prepare_geometry_replay_with_revisions(&[(model, 1)], false, REPLAY_LIMIT, None);
    assert_eq!(continued.selected_primitive_count(), 44);
    assert_selection_work_bounded(&projector, 1, continued.selected_primitive_count());

    reset_selection_counters(&projector);
    let restarted =
        projector.prepare_geometry_replay_with_revisions(&[(model, 2)], false, REPLAY_LIMIT, None);
    assert_eq!(restarted.target_keys(), first.target_keys());
    assert!(restarted.truncated());
    assert_eq!(
        assert_selection_work_bounded(&projector, 1, restarted.selected_primitive_count()),
        REPLAY_LIMIT + 1
    );
}

#[test]
fn render_scene_resource_geometry_replay_bounded_targets_restart_changed_scope() {
    let mut renderer = MeshRenderer::default();
    let model = UntypedResourceHandle::new(renderer.model.id(), ResourceKind::Model);
    let mesh = ResourceHandle::<MeshMarker>::new(ResourceId::from_stable_label("bounded/scope"));
    let resource = UntypedResourceHandle::new(mesh.id(), ResourceKind::Mesh);
    renderer.primitives.push(MeshRendererPrimitiveBinding {
        mesh,
        material: renderer.material,
    });
    let (mut projector, _) = projector_with_renderers((0..300).map(|_| renderer.clone()));
    let first = projector.prepare_geometry_replay(&[model], false, REPLAY_LIMIT, None);
    projector.finish_geometry_replay_selection(&first);

    // Both revision maps are empty, so only exact scope equality can permit continuation.
    reset_selection_counters(&projector);
    let restarted =
        projector.prepare_geometry_replay(&[resource, model], false, REPLAY_LIMIT, None);
    assert_eq!(restarted.target_keys(), first.target_keys());
    assert!(restarted.truncated());
    assert_eq!(
        assert_selection_work_bounded(&projector, 2, restarted.selected_primitive_count()),
        2 * (REPLAY_LIMIT + 1)
    );
}
