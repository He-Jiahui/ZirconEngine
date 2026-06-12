use crate::core::framework::render::{
    CorePipelineKind, FallbackSkyboxKind, GeometryExtract, GeometryPhaseInput,
    PreviewEnvironmentExtract, RenderFrameExtract, RenderFramePhaseQueueSummaryPhaseCount,
    RenderFramePhaseQueueSummaryPhaseOrderSpan, RenderMaterialAlphaMode, RenderOverlayExtract,
    RenderPhase, RenderPhaseItem, RenderPhaseMeshSource, RenderPhaseQueue,
    RenderPhaseQueueSummaryPhaseOrderSpan, RenderPhaseSortKey, RenderSceneGeometryExtract,
    RenderSceneSnapshot, SpriteExtract, SpritePhaseExtractInput, RENDER_PHASES_BY_QUEUE_ORDER,
};
use crate::core::framework::scene::WorldHandle;
use crate::core::math::Vec4;

#[test]
fn render_phase_queue_summary_reports_phase_counts_and_ordering_bounds() {
    let item = |entity, phase, sort_key| RenderPhaseItem {
        entity,
        phase,
        sort_key: RenderPhaseSortKey::new(sort_key),
        mesh_source: RenderPhaseMeshSource::MeshIndex(entity as usize),
    };
    let queue = RenderPhaseQueue::new(vec![
        item(30, RenderPhase::Transparent3d, 10),
        item(10, RenderPhase::Shadow, 5),
        item(20, RenderPhase::Opaque3d, 1),
        item(21, RenderPhase::Opaque3d, 2),
    ]);
    let summary = queue.summary();
    let serialized_summary = serde_json::to_string(&summary).unwrap();
    assert!(serialized_summary.contains("\"diagnostic_name\":\"opaque-2d+opaque-3d\""));
    assert_eq!(
        serde_json::from_str::<crate::core::framework::render::RenderPhaseQueueSummary>(
            &serialized_summary
        )
        .unwrap(),
        summary
    );

    assert_eq!(summary.item_count, 4);
    assert_eq!(summary.count_for_phase(RenderPhase::Prepass), 0);
    assert_eq!(summary.count_for_phase(RenderPhase::Shadow), 1);
    assert_eq!(summary.count_for_phase(RenderPhase::Opaque2d), 0);
    assert_eq!(summary.count_for_phase(RenderPhase::Opaque3d), 2);
    assert_eq!(summary.count_for_phase(RenderPhase::Transparent3d), 1);
    assert_eq!(
        summary
            .phase_counts
            .iter()
            .filter_map(|count| (count.item_count > 0).then_some((count.phase, count.item_count)))
            .collect::<Vec<_>>(),
        vec![
            (RenderPhase::Shadow, 1),
            (RenderPhase::Opaque3d, 2),
            (RenderPhase::Transparent3d, 1),
        ]
    );
    assert_eq!(
        summary
            .active_phase_counts()
            .map(|count| (count.diagnostic_name(), count.item_count))
            .collect::<Vec<_>>(),
        vec![("shadow", 1), ("opaque-3d", 2), ("transparent-3d", 1)]
    );
    assert_eq!(
        summary
            .phase_counts
            .iter()
            .map(|count| count.phase)
            .collect::<Vec<_>>(),
        RENDER_PHASES_BY_QUEUE_ORDER.to_vec()
    );
    assert!(summary
        .phase_counts
        .iter()
        .all(|count| count.phase_order == count.phase.queue_order()));
    assert_eq!(
        summary
            .phase_count_row_for_phase(RenderPhase::Opaque3d)
            .unwrap()
            .diagnostic_name(),
        "opaque-3d"
    );
    assert_eq!(
        summary
            .phase_count_row_for_phase(RenderPhase::Ui)
            .unwrap()
            .item_count,
        0
    );
    assert_eq!(
        summary.count_for_phase_order(RenderPhase::Shadow.queue_order()),
        1
    );
    assert_eq!(
        summary.count_for_phase_order(RenderPhase::Opaque3d.queue_order()),
        2
    );
    assert_eq!(
        summary.count_for_phase_order(RenderPhase::Transparent3d.queue_order()),
        1
    );
    assert_eq!(
        summary
            .active_phase_order_spans()
            .map(|span| (span.diagnostic_name(), span.item_count))
            .collect::<Vec<_>>(),
        vec![
            ("shadow", 1),
            ("opaque-2d+opaque-3d", 2),
            ("transparent-2d+transparent-3d", 1)
        ]
    );
    assert_eq!(
        summary.span_for_phase_order(RenderPhase::Shadow.queue_order()),
        Some(&RenderPhaseQueueSummaryPhaseOrderSpan {
            phase_order: RenderPhase::Shadow.queue_order(),
            diagnostic_name: "shadow".to_string(),
            phases: vec![RenderPhase::Shadow],
            item_count: 1,
            start_index: Some(0),
            end_index_exclusive: Some(1),
            first_ordering_key: Some(queue.items[0].ordering_key()),
            last_ordering_key: Some(queue.items[0].ordering_key()),
        })
    );
    assert_eq!(
        summary.span_for_phase_order(RenderPhase::Opaque3d.queue_order()),
        Some(&RenderPhaseQueueSummaryPhaseOrderSpan {
            phase_order: RenderPhase::Opaque3d.queue_order(),
            diagnostic_name: "opaque-2d+opaque-3d".to_string(),
            phases: vec![RenderPhase::Opaque2d, RenderPhase::Opaque3d],
            item_count: 2,
            start_index: Some(1),
            end_index_exclusive: Some(3),
            first_ordering_key: Some(queue.items[1].ordering_key()),
            last_ordering_key: Some(queue.items[2].ordering_key()),
        })
    );
    assert_eq!(
        summary.span_for_phase(RenderPhase::Opaque2d),
        summary.span_for_phase(RenderPhase::Opaque3d)
    );
    assert_eq!(
        summary.span_for_phase(RenderPhase::Transparent2d),
        summary.span_for_phase(RenderPhase::Transparent3d)
    );
    assert_eq!(
        summary.span_for_queue_index(0),
        summary.span_for_phase(RenderPhase::Shadow)
    );
    assert_eq!(
        summary.span_for_queue_index(1),
        summary.span_for_phase(RenderPhase::Opaque3d)
    );
    assert_eq!(
        summary.span_for_queue_index(2),
        summary.span_for_phase(RenderPhase::Opaque2d)
    );
    assert_eq!(
        summary.span_for_queue_index(3),
        summary.span_for_phase(RenderPhase::Transparent3d)
    );
    assert_eq!(summary.span_for_queue_index(4), None);
    assert_eq!(
        summary
            .span_for_phase_order(RenderPhase::Prepass.queue_order())
            .unwrap()
            .phases,
        vec![RenderPhase::Prepass]
    );
    assert_eq!(RenderPhase::Opaque2d.diagnostic_name(), "opaque-2d");
    assert_eq!(RenderPhase::Opaque3d.diagnostic_name(), "opaque-3d");
    assert_eq!(
        summary
            .span_for_phase(RenderPhase::Opaque3d)
            .unwrap()
            .diagnostic_name(),
        "opaque-2d+opaque-3d"
    );
    assert_eq!(
        summary
            .span_for_phase(RenderPhase::Prepass)
            .unwrap()
            .diagnostic_name(),
        "prepass"
    );
    assert_eq!(
        summary
            .span_for_phase_order(RenderPhase::Prepass.queue_order())
            .unwrap()
            .start_index,
        None
    );
    assert_eq!(
        summary.first_ordering_key,
        Some(queue.items[0].ordering_key())
    );
    assert_eq!(
        summary.last_ordering_key,
        Some(queue.items[3].ordering_key())
    );

    let empty_summary = RenderPhaseQueue::new(Vec::new()).summary();
    assert_eq!(empty_summary.item_count, 0);
    assert_eq!(empty_summary.first_ordering_key, None);
    assert_eq!(empty_summary.last_ordering_key, None);
}

#[test]
fn geometry_extract_phase_queue_summary_reports_sorted_bounds() {
    let extract = GeometryExtract::from_meshes_and_phase_inputs(
        CorePipelineKind::Core3d,
        Vec::new(),
        vec![
            GeometryPhaseInput::new(30, 0, RenderMaterialAlphaMode::Opaque, 10.0)
                .with_render_queue(2_000)
                .with_material_queue(0),
            GeometryPhaseInput::new(10, 1, RenderMaterialAlphaMode::Opaque, 1.0)
                .with_render_queue(1_000)
                .with_material_queue(50),
            GeometryPhaseInput::new(20, 2, RenderMaterialAlphaMode::Opaque, 0.0)
                .with_render_queue(2_000)
                .with_material_queue(-10)
                .with_order_in_layer(5),
        ],
    );

    let summary = extract.phase_queue_summary();
    assert_eq!(summary.item_count, extract.phase_queue.items.len());
    assert_eq!(
        summary.count_for_phase(RenderPhase::Opaque3d),
        extract.phase_queue.items.len()
    );
    let opaque_span = summary.span_for_phase(RenderPhase::Opaque3d).unwrap();
    assert_eq!(opaque_span.start_index, Some(0));
    assert_eq!(
        opaque_span.end_index_exclusive,
        Some(extract.phase_queue.items.len())
    );
    assert_eq!(
        opaque_span.first_ordering_key,
        extract
            .phase_queue
            .items
            .first()
            .map(RenderPhaseItem::ordering_key)
    );
    assert_eq!(
        opaque_span.last_ordering_key,
        extract
            .phase_queue
            .items
            .last()
            .map(RenderPhaseItem::ordering_key)
    );
}

#[test]
fn sprite_extract_phase_queue_summary_reports_core2d_phase_counts() {
    let extract = SpriteExtract::from_sprites_and_phase_inputs(
        CorePipelineKind::Core2d,
        Vec::new(),
        vec![
            SpritePhaseExtractInput::new(30, 0, RenderMaterialAlphaMode::Blend, 2, 2.0),
            SpritePhaseExtractInput::new(10, 1, RenderMaterialAlphaMode::Opaque, 0, 1.0),
            SpritePhaseExtractInput::new(20, 2, RenderMaterialAlphaMode::Blend, 1, 4.0),
        ],
    );

    let summary = extract.phase_queue_summary();
    assert_eq!(summary.item_count, extract.phase_queue.items.len());
    assert_eq!(summary.count_for_phase(RenderPhase::Opaque2d), 1);
    assert_eq!(summary.count_for_phase(RenderPhase::Transparent2d), 2);
    assert_eq!(
        summary.count_for_phase_order(RenderPhase::Transparent2d.queue_order()),
        2
    );

    let transparent_span = summary.span_for_phase(RenderPhase::Transparent2d).unwrap();
    assert_eq!(
        transparent_span.phases,
        vec![RenderPhase::Transparent2d, RenderPhase::Transparent3d]
    );
    assert_eq!(transparent_span.start_index, Some(1));
    assert_eq!(transparent_span.end_index_exclusive, Some(3));
    assert_eq!(
        transparent_span.first_ordering_key,
        extract
            .phase_queue
            .items
            .get(1)
            .map(RenderPhaseItem::ordering_key)
    );
    assert_eq!(
        transparent_span.last_ordering_key,
        extract
            .phase_queue
            .items
            .get(2)
            .map(RenderPhaseItem::ordering_key)
    );
}

#[test]
fn render_frame_phase_queue_summary_merges_geometry_and_sprite_counts() {
    let mut frame = RenderFrameExtract::from_snapshot(
        WorldHandle::new(5).into(),
        RenderSceneSnapshot {
            scene: RenderSceneGeometryExtract {
                camera: Default::default(),
                meshes: Vec::new(),
                directional_lights: Vec::new(),
                point_lights: Vec::new(),
                spot_lights: Vec::new(),
                ambient_lights: Vec::new(),
                rect_lights: Vec::new(),
            },
            overlays: RenderOverlayExtract::default(),
            preview: PreviewEnvironmentExtract {
                lighting_enabled: false,
                skybox_enabled: false,
                fallback_skybox: FallbackSkyboxKind::None,
                clear_color: Vec4::ZERO,
            },
            virtual_geometry_debug: None,
        },
    );
    frame.geometry = GeometryExtract::from_meshes_and_phase_inputs(
        CorePipelineKind::Core3d,
        Vec::new(),
        vec![
            GeometryPhaseInput::new(10, 0, RenderMaterialAlphaMode::Opaque, 1.0),
            GeometryPhaseInput::new(20, 1, RenderMaterialAlphaMode::Blend, 10.0),
        ],
    );
    frame.sprites = SpriteExtract::from_sprites_and_phase_inputs(
        CorePipelineKind::Core2d,
        Vec::new(),
        vec![
            SpritePhaseExtractInput::new(30, 0, RenderMaterialAlphaMode::Opaque, 0, 0.0),
            SpritePhaseExtractInput::new(40, 1, RenderMaterialAlphaMode::Blend, 2, 2.0),
        ],
    );

    let summary = frame.phase_queue_summary();
    let serialized_summary = serde_json::to_string(&summary).unwrap();
    assert!(serialized_summary.contains("\"diagnostic_name\":\"opaque-2d+opaque-3d\""));
    assert_eq!(
        serde_json::from_str::<crate::core::framework::render::RenderFramePhaseQueueSummary>(
            &serialized_summary
        )
        .unwrap(),
        summary
    );
    assert_eq!(summary.geometry, frame.geometry.phase_queue_summary());
    assert_eq!(summary.sprites, frame.sprites.phase_queue_summary());
    assert_eq!(
        summary.total_item_count,
        frame.geometry.phase_queue.items.len() + frame.sprites.phase_queue.items.len()
    );
    assert_eq!(
        summary.geometry_first_ordering_key,
        frame
            .geometry
            .phase_queue
            .items
            .first()
            .map(RenderPhaseItem::ordering_key)
    );
    assert_eq!(
        summary.geometry_last_ordering_key,
        frame
            .geometry
            .phase_queue
            .items
            .last()
            .map(RenderPhaseItem::ordering_key)
    );
    assert_eq!(
        summary.sprite_first_ordering_key,
        frame
            .sprites
            .phase_queue
            .items
            .first()
            .map(RenderPhaseItem::ordering_key)
    );
    assert_eq!(
        summary.sprite_last_ordering_key,
        frame
            .sprites
            .phase_queue
            .items
            .last()
            .map(RenderPhaseItem::ordering_key)
    );
    assert_eq!(summary.count_for_phase(RenderPhase::Opaque3d), 1);
    assert_eq!(summary.count_for_phase(RenderPhase::Transparent3d), 1);
    assert_eq!(summary.count_for_phase(RenderPhase::Opaque2d), 1);
    assert_eq!(summary.count_for_phase(RenderPhase::Transparent2d), 1);
    assert_eq!(
        summary.count_for_phase_order(RenderPhase::Opaque3d.queue_order()),
        2
    );
    assert_eq!(
        summary.count_for_phase_order(RenderPhase::Transparent2d.queue_order()),
        2
    );
    assert_eq!(
        summary
            .phase_counts
            .iter()
            .map(|count| count.phase)
            .collect::<Vec<_>>(),
        RENDER_PHASES_BY_QUEUE_ORDER.to_vec()
    );
    assert!(
        summary
            .phase_counts
            .iter()
            .all(|count| count.total_item_count
                == count.geometry_item_count + count.sprite_item_count)
    );
    assert_eq!(
        summary
            .phase_counts
            .iter()
            .filter_map(|count| (count.total_item_count > 0).then_some((
                count.phase,
                count.geometry_item_count,
                count.sprite_item_count,
                count.total_item_count,
            )))
            .collect::<Vec<_>>(),
        vec![
            (RenderPhase::Opaque2d, 0, 1, 1),
            (RenderPhase::Opaque3d, 1, 0, 1),
            (RenderPhase::Transparent2d, 0, 1, 1),
            (RenderPhase::Transparent3d, 1, 0, 1),
        ]
    );
    assert_eq!(
        summary
            .active_phase_counts()
            .map(|count| (count.diagnostic_name(), count.total_item_count))
            .collect::<Vec<_>>(),
        vec![
            ("opaque-2d", 1),
            ("opaque-3d", 1),
            ("transparent-2d", 1),
            ("transparent-3d", 1),
        ]
    );
    assert_eq!(
        summary.phase_count_row_for_phase(RenderPhase::Opaque3d),
        Some(&RenderFramePhaseQueueSummaryPhaseCount {
            phase: RenderPhase::Opaque3d,
            diagnostic_name: "opaque-3d".to_string(),
            phase_order: RenderPhase::Opaque3d.queue_order(),
            geometry_item_count: 1,
            sprite_item_count: 0,
            total_item_count: 1,
        })
    );
    assert_eq!(
        summary
            .phase_count_row_for_phase(RenderPhase::Opaque2d)
            .unwrap()
            .diagnostic_name(),
        "opaque-2d"
    );
    assert_eq!(
        summary
            .phase_order_spans
            .iter()
            .filter_map(|span| (span.total_item_count > 0).then_some((
                span.phase_order,
                span.phases.clone(),
                span.geometry_item_count,
                span.sprite_item_count,
                span.total_item_count,
            )))
            .collect::<Vec<_>>(),
        vec![
            (
                RenderPhase::Opaque3d.queue_order(),
                vec![RenderPhase::Opaque2d, RenderPhase::Opaque3d],
                1,
                1,
                2,
            ),
            (
                RenderPhase::Transparent3d.queue_order(),
                vec![RenderPhase::Transparent2d, RenderPhase::Transparent3d],
                1,
                1,
                2,
            ),
        ]
    );
    assert_eq!(
        summary
            .active_phase_order_spans()
            .map(|span| (span.diagnostic_name(), span.total_item_count))
            .collect::<Vec<_>>(),
        vec![
            ("opaque-2d+opaque-3d", 2),
            ("transparent-2d+transparent-3d", 2),
        ]
    );
    assert_eq!(
        summary.phase_order_span_for_phase(RenderPhase::Opaque2d),
        summary.phase_order_span_for_phase(RenderPhase::Opaque3d)
    );
    assert_eq!(
        summary.phase_order_span_for_geometry_queue_index(0),
        summary.phase_order_span_for_phase(RenderPhase::Opaque3d)
    );
    assert_eq!(
        summary.phase_order_span_for_geometry_queue_index(1),
        summary.phase_order_span_for_phase(RenderPhase::Transparent3d)
    );
    assert_eq!(summary.phase_order_span_for_geometry_queue_index(2), None);
    assert_eq!(
        summary.phase_order_span_for_sprite_queue_index(0),
        summary.phase_order_span_for_phase(RenderPhase::Opaque2d)
    );
    assert_eq!(
        summary.phase_order_span_for_sprite_queue_index(1),
        summary.phase_order_span_for_phase(RenderPhase::Transparent2d)
    );
    assert_eq!(summary.phase_order_span_for_sprite_queue_index(2), None);
    assert_eq!(
        summary.phase_order_span_for_phase_order(RenderPhase::Opaque3d.queue_order()),
        Some(&RenderFramePhaseQueueSummaryPhaseOrderSpan {
            phase_order: RenderPhase::Opaque3d.queue_order(),
            diagnostic_name: "opaque-2d+opaque-3d".to_string(),
            phases: vec![RenderPhase::Opaque2d, RenderPhase::Opaque3d],
            geometry_item_count: 1,
            sprite_item_count: 1,
            total_item_count: 2,
            geometry_start_index: Some(0),
            geometry_end_index_exclusive: Some(1),
            geometry_first_ordering_key: frame
                .geometry
                .phase_queue
                .items
                .first()
                .map(RenderPhaseItem::ordering_key),
            geometry_last_ordering_key: frame
                .geometry
                .phase_queue
                .items
                .first()
                .map(RenderPhaseItem::ordering_key),
            sprite_start_index: Some(0),
            sprite_end_index_exclusive: Some(1),
            sprite_first_ordering_key: frame
                .sprites
                .phase_queue
                .items
                .first()
                .map(RenderPhaseItem::ordering_key),
            sprite_last_ordering_key: frame
                .sprites
                .phase_queue
                .items
                .first()
                .map(RenderPhaseItem::ordering_key),
        })
    );
    let transparent_span = summary
        .phase_order_span_for_phase(RenderPhase::Transparent3d)
        .unwrap();
    assert_eq!(transparent_span.geometry_start_index, Some(1));
    assert_eq!(transparent_span.geometry_end_index_exclusive, Some(2));
    assert_eq!(
        transparent_span.geometry_first_ordering_key,
        frame
            .geometry
            .phase_queue
            .items
            .get(1)
            .map(RenderPhaseItem::ordering_key)
    );
    assert_eq!(
        transparent_span.geometry_last_ordering_key,
        frame
            .geometry
            .phase_queue
            .items
            .get(1)
            .map(RenderPhaseItem::ordering_key)
    );
    assert_eq!(transparent_span.sprite_start_index, Some(1));
    assert_eq!(transparent_span.sprite_end_index_exclusive, Some(2));
    assert_eq!(
        transparent_span.sprite_first_ordering_key,
        frame
            .sprites
            .phase_queue
            .items
            .get(1)
            .map(RenderPhaseItem::ordering_key)
    );
    assert_eq!(
        transparent_span.sprite_last_ordering_key,
        frame
            .sprites
            .phase_queue
            .items
            .get(1)
            .map(RenderPhaseItem::ordering_key)
    );
    let empty_prepass_span = summary
        .phase_order_span_for_phase(RenderPhase::Prepass)
        .unwrap();
    assert_eq!(empty_prepass_span.diagnostic_name(), "prepass");
    assert_eq!(empty_prepass_span.geometry_start_index, None);
    assert_eq!(empty_prepass_span.geometry_end_index_exclusive, None);
    assert_eq!(empty_prepass_span.geometry_first_ordering_key, None);
    assert_eq!(empty_prepass_span.geometry_last_ordering_key, None);
    assert_eq!(empty_prepass_span.sprite_start_index, None);
    assert_eq!(empty_prepass_span.sprite_end_index_exclusive, None);
    assert_eq!(empty_prepass_span.sprite_first_ordering_key, None);
    assert_eq!(empty_prepass_span.sprite_last_ordering_key, None);

    let empty_frame = RenderFrameExtract::from_snapshot(
        WorldHandle::new(6).into(),
        RenderSceneSnapshot {
            scene: RenderSceneGeometryExtract {
                camera: Default::default(),
                meshes: Vec::new(),
                directional_lights: Vec::new(),
                point_lights: Vec::new(),
                spot_lights: Vec::new(),
                ambient_lights: Vec::new(),
                rect_lights: Vec::new(),
            },
            overlays: RenderOverlayExtract::default(),
            preview: PreviewEnvironmentExtract {
                lighting_enabled: false,
                skybox_enabled: false,
                fallback_skybox: FallbackSkyboxKind::None,
                clear_color: Vec4::ZERO,
            },
            virtual_geometry_debug: None,
        },
    );
    let empty_summary = empty_frame.phase_queue_summary();
    assert_eq!(empty_summary.total_item_count, 0);
    assert_eq!(empty_summary.geometry_first_ordering_key, None);
    assert_eq!(empty_summary.geometry_last_ordering_key, None);
    assert_eq!(empty_summary.sprite_first_ordering_key, None);
    assert_eq!(empty_summary.sprite_last_ordering_key, None);
}
