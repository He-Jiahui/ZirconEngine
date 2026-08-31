use super::*;

#[test]
fn particle_extract_filters_dynamic_component_candidates_before_scene_state_queries() {
    let source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("scene")
            .join("world")
            .join("render_particles.rs"),
    )
    .unwrap();
    let collector = source
        .split("pub(super) fn collect_render_particles")
        .nth(1)
        .and_then(|text| text.split("#[derive(Clone").next())
        .expect("read particle collector");
    let candidate_lookup = collector
        .find("let particle_values =")
        .expect("particle collector should snapshot relevant dynamic component references first");
    let scene_state_lookup = collector
        .find("self.active_in_hierarchy(entity)")
        .expect("particle collector should filter active scene entities");

    assert!(
        candidate_lookup < scene_state_lookup,
        "entities without particle/HUD components must not pay active hierarchy and layer queries"
    );
}

#[test]
fn render_frame_extract_collects_dynamic_particle_sprites_by_camera_layers() {
    let mut world = World::empty();
    let camera = spawn_camera_on_layer(&mut world, 0b0010);
    world.set_active_camera(camera);
    let visible = world
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    let hidden = world
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    world.set_render_layer_mask(visible, 0b0010).unwrap();
    world.set_render_layer_mask(hidden, 0b0100).unwrap();
    world
        .set_dynamic_component(
            visible,
            "render.particle_sprites",
            serde_json::json!({
                "style": "blood_flame_haste_shield",
                "sprites": [{
                    "position": [1.0, 2.0, 3.0],
                    "size": 0.45,
                    "rotation": 0.25,
                    "color": [1.0, 0.2, 0.1, 0.75],
                    "intensity": 1.8
                }]
            }),
        )
        .unwrap();
    world
        .set_dynamic_component(
            hidden,
            "render.particle_sprites",
            serde_json::json!({
                "sprites": [{
                    "position": [6.0, 2.0, 3.0],
                    "size": 0.8,
                    "color": [0.1, 0.9, 1.0, 1.0],
                    "intensity": 3.0
                }]
            }),
        )
        .unwrap();

    let extract = world.build_prepared_render_frame_extract(&RenderExtractContext::new(
        RenderWorldSnapshotHandle::new(706),
        SceneViewportExtractRequest::default(),
    ));

    assert_eq!(extract.particles.emitters, vec![visible]);
    assert_eq!(extract.particles.sprites.len(), 1);
    let sprite = &extract.particles.sprites[0];
    assert_eq!(sprite.entity, visible);
    assert_eq!(sprite.position, Vec3::new(1.0, 2.0, 3.0));
    assert_eq!(sprite.size, 0.45);
    assert_eq!(sprite.rotation, 0.25);
    assert_eq!(sprite.color, Vec4::new(1.0, 0.2, 0.1, 0.75));
    assert_eq!(sprite.intensity, 1.8);
    assert_eq!(extract.particles.bounds.len(), 1);
    assert_eq!(extract.particles.bounds[0].entity, visible);
    assert_eq!(
        extract.particles.sort_camera_position,
        Some(extract.view.camera.transform.translation)
    );
    assert!(extract.visibility.dynamic_entities.contains(&visible));
    assert!(!extract.visibility.dynamic_entities.contains(&hidden));
}

#[test]
fn render_frame_extract_collects_dynamic_particle_gpu_frames_by_camera_layers() {
    let mut world = World::empty();
    let camera = spawn_camera_on_layer(&mut world, 0b0010);
    world.set_active_camera(camera);
    let visible = world
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    let hidden = world
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    world.set_render_layer_mask(visible, 0b0010).unwrap();
    world.set_render_layer_mask(hidden, 0b0100).unwrap();
    world
        .update_transform(
            visible,
            Transform::from_translation(Vec3::new(1.0, 2.0, 3.0)),
        )
        .unwrap();
    world
        .set_dynamic_component(
            visible,
            "render.particle_sprites",
            serde_json::json!({
                "gpu_frame": {
                    "alive_count": 5,
                    "spawned_total": 8,
                    "per_emitter_spawned": [3, 5],
                    "bounds": {
                        "center": [2.0, 2.0, 3.0],
                        "radius": 4.0
                    }
                }
            }),
        )
        .unwrap();
    world
        .set_dynamic_component(
            hidden,
            "render.particle_sprites",
            serde_json::json!({
                "gpu_frame": {
                    "alive_count": 11,
                    "spawned_total": 13,
                    "per_emitter_spawned": [13]
                }
            }),
        )
        .unwrap();

    let extract = world.build_prepared_render_frame_extract(&RenderExtractContext::new(
        RenderWorldSnapshotHandle::new(708),
        SceneViewportExtractRequest::default(),
    ));

    assert_eq!(extract.particles.emitters, vec![visible]);
    assert!(extract.particles.sprites.is_empty());
    assert_eq!(extract.particles.bounds.len(), 1);
    assert_eq!(extract.particles.bounds[0].entity, visible);
    assert_eq!(extract.particles.bounds[0].center, Vec3::new(1.0, 2.0, 3.0));
    assert_eq!(extract.particles.bounds[0].radius, 5.0);
    let gpu_frame = extract
        .particles
        .gpu_frame
        .expect("visible particle gpu frame should be projected");
    assert_eq!(gpu_frame.alive_count, 5);
    assert_eq!(gpu_frame.spawned_total, 8);
    assert_eq!(gpu_frame.per_emitter_spawned, vec![3, 5]);
    assert_eq!(gpu_frame.indirect_draw_args, [6, 5, 0, 0]);
    assert!(extract.visibility.dynamic_entities.contains(&visible));
    assert!(!extract.visibility.dynamic_entities.contains(&hidden));
}

#[test]
fn render_frame_extract_collects_world_hud_health_bars_as_scene_particles() {
    let mut world = World::empty();
    let camera = spawn_camera_on_layer(&mut world, 0b0010);
    world.set_active_camera(camera);
    let visible = world
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    let hidden = world
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    world.set_render_layer_mask(visible, 0b0010).unwrap();
    world.set_render_layer_mask(hidden, 0b0100).unwrap();
    world
        .set_dynamic_component(
            visible,
            "render.world_hud_bars",
            serde_json::json!({
                "bars": [{
                    "position": [2.0, 3.5, 4.0],
                    "width": 1.2,
                    "height": 0.12,
                    "ratio": 0.5,
                    "segments": 4,
                    "back_color": [0.04, 0.03, 0.04, 0.7],
                    "fill_color": [0.2, 0.9, 0.35, 0.88],
                    "intensity": 1.25
                }]
            }),
        )
        .unwrap();
    world
        .set_dynamic_component(
            hidden,
            "render.world_hud_bars",
            serde_json::json!({
                "bars": [{
                    "position": [6.0, 3.5, 4.0],
                    "ratio": 1.0,
                    "segments": 4
                }]
            }),
        )
        .unwrap();

    let extract = world.build_prepared_render_frame_extract(&RenderExtractContext::new(
        RenderWorldSnapshotHandle::new(707),
        SceneViewportExtractRequest::default(),
    ));

    assert_eq!(extract.particles.emitters, vec![visible]);
    assert_eq!(
        extract.particles.sprites.len(),
        2,
        "world HUD health should render as one background billboard and one fill billboard"
    );
    assert_eq!(
        extract.particles.sprites[0].color,
        Vec4::new(0.04, 0.03, 0.04, 0.7),
        "background should be submitted before the fill for same-entity HUD bars"
    );
    assert_eq!(extract.particles.sprites[0].size, 0.12);
    assert!(
        (extract.particles.sprites[0].aspect_ratio - 10.0).abs() <= 0.0001,
        "background width/height ratio should remain stable within f32 precision"
    );
    assert_eq!(extract.particles.sprites[0].billboard_offset, Vec2::ZERO);
    assert_eq!(
        extract.particles.sprites[1].color,
        Vec4::new(0.2, 0.9, 0.35, 0.88)
    );
    assert!(
        (extract.particles.sprites[1].size - 0.0864).abs() <= 0.0001,
        "fill height should be inset from the background"
    );
    assert!(
        (extract.particles.sprites[1].aspect_ratio - (0.6 / 0.0864)).abs() <= 0.0001,
        "fill width should encode the health ratio"
    );
    assert_eq!(
        extract.particles.sprites[1].billboard_offset,
        Vec2::new(-0.3, 0.0),
        "fill should stay left-aligned inside the background bar"
    );
    assert_eq!(
        extract
            .particles
            .sprites
            .iter()
            .filter(|sprite| sprite.color == Vec4::new(0.04, 0.03, 0.04, 0.7))
            .count(),
        1,
        "world HUD bar should emit one background billboard"
    );
    assert_eq!(
        extract
            .particles
            .sprites
            .iter()
            .filter(|sprite| sprite.color == Vec4::new(0.2, 0.9, 0.35, 0.88))
            .count(),
        1,
        "world HUD bar should emit one filled billboard from its health ratio"
    );
    assert!(
        extract
            .particles
            .sprites
            .iter()
            .all(|sprite| sprite.entity == visible && sprite.position.y == 3.5)
    );
    assert!(extract.visibility.dynamic_entities.contains(&visible));
    assert!(!extract.visibility.dynamic_entities.contains(&hidden));
}
