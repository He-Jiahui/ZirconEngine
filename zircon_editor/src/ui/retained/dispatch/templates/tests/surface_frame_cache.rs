use super::{update_hit_route_key, SurfaceFrameNode, SurfaceFrameSignature};

fn signature_with_hit_controls(values: &[Option<&str>]) -> SurfaceFrameSignature {
    SurfaceFrameSignature {
        width_bits: 1280.0_f32.to_bits(),
        height_bits: 28.0_f32.to_bits(),
        nodes: values
            .iter()
            .enumerate()
            .map(|(index, hit_control_id)| SurfaceFrameNode {
                projection_control_id: format!("projection-{index}"),
                hit_control_id: hit_control_id.map(str::to_owned),
                component: "button".to_owned(),
                frame: Default::default(),
            })
            .collect(),
    }
}

#[test]
fn hit_route_key_updates_reuse_capacity_and_clear_cleanly() {
    let mut cached = None;
    update_hit_route_key(&mut cached, Some(&["first", "second"]));
    let values = cached.as_ref().expect("route key must be stored");
    let first_capacity = values.capacity();
    let first_value_capacity = values[0].capacity();
    let second_value_capacity = values[1].capacity();
    assert!(first_capacity >= 2);

    update_hit_route_key(&mut cached, Some(&["again", "route"]));
    let values = cached.as_ref().expect("route key remains stored");
    assert_eq!(values.len(), 2);
    assert_eq!(values[0], "again");
    assert_eq!(values[1], "route");
    assert_eq!(
        values.capacity(),
        first_capacity,
        "same-shape route updates should reuse the retained allocation"
    );
    assert_eq!(
        values[0].capacity(),
        first_value_capacity,
        "same-shape route updates should reuse each retained string buffer"
    );
    assert_eq!(
        values[1].capacity(),
        second_value_capacity,
        "same-shape route updates should reuse each retained string buffer"
    );

    update_hit_route_key(
        &mut cached,
        Some(&[
            "a-route-value-long-enough-to-grow-the-retained-string",
            "another-route-value-long-enough-to-grow-the-retained-string",
        ]),
    );
    let values = cached.as_ref().expect("grown route strings remain stored");
    let first_long_capacity = values[0].capacity();
    let second_long_capacity = values[1].capacity();
    assert!(first_long_capacity >= values[0].len());
    assert!(second_long_capacity >= values[1].len());

    update_hit_route_key(
        &mut cached,
        Some(&[
            "a-route-value-long-enough-to-grow-the-retained-string-again",
            "another-route-value-long-enough-to-grow-the-retained-string-again",
        ]),
    );
    let values = cached.as_ref().expect("route strings remain stored");
    assert_eq!(
        values[0].capacity(),
        first_long_capacity,
        "long route updates should reuse each retained string buffer"
    );
    assert_eq!(
        values[1].capacity(),
        second_long_capacity,
        "long route updates should reuse each retained string buffer"
    );

    update_hit_route_key(&mut cached, Some(&["again", "route", "third"]));
    let values = cached.as_ref().expect("grown route key remains stored");
    assert_eq!(values.len(), 3);
    assert_eq!(values[2], "third");
    assert!(values.capacity() >= 3);

    update_hit_route_key(&mut cached, Some(&["short"]));
    let values = cached.as_ref().expect("shrunk route key remains stored");
    assert_eq!(values.len(), 1);
    assert_eq!(values[0], "short");

    update_hit_route_key(&mut cached, None);
    assert!(cached.is_none());
}

#[test]
fn hit_control_remap_updates_signature_in_place_and_reports_visits() {
    let mut signature = signature_with_hit_controls(&[Some("old"), None]);
    let (changed, visits) = signature.remap_hit_control_ids(&mut |control_id| {
        Some(match control_id {
            "projection-0" => "new".to_owned(),
            _ => "second".to_owned(),
        })
    });
    assert!(changed);
    assert_eq!(visits, 2);
    assert_eq!(signature.nodes[0].hit_control_id.as_deref(), Some("new"));
    assert_eq!(signature.nodes[1].hit_control_id.as_deref(), Some("second"));

    let (changed_again, visits_again) = signature.remap_hit_control_ids(&mut |control_id| {
        Some(match control_id {
            "projection-0" => "new".to_owned(),
            _ => "second".to_owned(),
        })
    });
    assert!(!changed_again);
    assert_eq!(visits_again, 2);
}

#[test]
fn real_toolbar_ancestor_clip_only_republishes_cached_hits() {
    use super::ViewportToolbarSurfaceFrameCache;
    use crate::ui::control::EditorUiControlService;
    use crate::ui::template_runtime::EditorUiHostRuntime;
    use std::sync::Arc;
    use zircon_runtime::ui::surface::hit_test_surface_frame;
    use zircon_runtime_interface::ui::layout::{UiPoint, UiSize};

    let _guard = crate::tests::support::env_lock().lock().unwrap();
    let document = "res://ui/editor/host/scene_viewport_toolbar.zui";
    let size = UiSize::new(1280.0, 28.0);
    let mut runtime = EditorUiHostRuntime::default();
    runtime.load_builtin_host_templates().unwrap();
    let mut document_projection = runtime.project_document(document).unwrap();
    runtime
        .register_projection_routes(
            &mut EditorUiControlService::default(),
            &mut document_projection,
        )
        .unwrap();
    let source_projection = |max_left| {
        let mut surface = runtime.build_shared_surface(document).unwrap();
        let left = surface
            .tree
            .nodes
            .values()
            .find_map(|node| {
                (node.template_metadata.as_ref()?.control_id.as_deref()
                    == Some("SceneViewportToolbarLeftGroup"))
                .then_some(node.node_id)
            })
            .expect("built-in toolbar scrollport");
        // Change a real layout constraint, never the test's clip/HitGrid entry.
        surface.tree.node_mut(left).unwrap().constraints.width.max = max_left;
        surface.compute_layout(size).unwrap();
        runtime
            .build_retained_host_projection_with_surface(&document_projection, &surface)
            .unwrap()
    };
    let before = source_projection(460.0);
    let partial = source_projection(76.0);
    let empty = source_projection(42.0);
    let hit = |frame: &zircon_runtime_interface::ui::surface::UiSurfaceFrame, p| {
        hit_test_surface_frame(frame, p)
            .top_entry(&frame.hit_grid)
            .is_some_and(|entry| entry.control_id.as_deref() == Some("SetPivotMode"))
    };
    let inside = UiPoint::new(68.0, 14.0);
    let outside = UiPoint::new(84.0, 14.0);
    let old_inputs = before
        .nodes
        .iter()
        .filter(|node| !node.routes.is_empty() && !node.disabled && node.control_id.is_some());
    for next in [&partial, &empty] {
        let new_inputs = next
            .nodes
            .iter()
            .filter(|node| !node.routes.is_empty() && !node.disabled && node.control_id.is_some());
        assert!(
            old_inputs
                .clone()
                .map(|node| (&node.control_id, &node.component, node.frame))
                .eq(new_inputs.map(|node| (&node.control_id, &node.component, node.frame))),
            "CACHE_ADMISSION: every raw mapped input and its ordering must stay fixed"
        );
    }
    let source_before = before.source_surface_frame.as_ref().unwrap();
    let source_partial = partial.source_surface_frame.as_ref().unwrap();
    let source_empty = empty.source_surface_frame.as_ref().unwrap();
    assert!(hit(source_before, inside) && hit(source_before, outside));
    assert!(hit(source_partial, inside) && !hit(source_partial, outside));
    assert!(!hit(source_empty, inside) && !hit(source_empty, outside));

    let mut cache = ViewportToolbarSurfaceFrameCache::default();
    let key = "scene:source-clip-only";
    let map = |id: &str| Some(id.to_owned());
    let original = cache.resolve(&before, key, size, Some(&["stable"]), map);
    let clipped = cache.resolve(&partial, key, size, Some(&["stable"]), map);
    let reused = cache
        .resolve_if_layout_matches(key, size, Some(&["stable"]), map)
        .unwrap();
    let gone = cache.resolve(&empty, key, size, Some(&["stable"]), map);
    eprintln!("SCENE_TOOLBAR_OUTER_CLIP_ONLY reuse_old={} reuse_partial={} original={:?} partial={:?} empty={:?}",
        Arc::ptr_eq(&original, &clipped), Arc::ptr_eq(&clipped, &reused),
        [hit(&original, inside), hit(&original, outside)], [hit(&clipped, inside), hit(&clipped, outside)],
        [hit(&gone, inside), hit(&gone, outside)]);
    assert!(hit(&original, inside) && hit(&original, outside));
    assert!(
        Arc::ptr_eq(&clipped, &reused),
        "stable prelayout fast path must keep reusing the current frame"
    );
    assert!(
        !Arc::ptr_eq(&original, &clipped)
            && hit(&clipped, inside)
            && !hit(&clipped, outside)
            && !Arc::ptr_eq(&clipped, &gone)
            && !hit(&gone, inside)
            && !hit(&gone, outside),
        "SCENE_TOOLBAR_OUTER_CLIP_ONLY: changed source clip must republish; empty must not hit"
    );
}
