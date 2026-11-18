//! Headless Move handle → retained preview → actual transaction/replay.
use crate::core::editing::command::EditorCommand;
use crate::core::editing::context::CoreEditContext;
use crate::core::editing::engine::{
    EditorTransactionEngine, HistoryContextId, HistoryStatus, MergeMode, TransactionId,
};
use crate::core::editing::interactive_transform::{
    selection_pivot_transform, InteractiveTransformKind, InteractiveTransformSession, PivotMode,
};
use crate::core::editing::selection::SceneSelection;
use crate::core::editor_message::DocumentId;
use crate::core::gateway::EditorRuntimeGatewayHandle;
use crate::scene::modes::SceneModeActivation;
use crate::scene::viewport::{
    GizmoAxis, SceneViewportController, TransformHandleKind, TransformSpace, ViewportCameraSnapshot,
};
use crate::ui::binding::ViewportCommand;
use zircon_runtime::scene::components::{LocalTransform, NodeKind, WorldMatrix};
use zircon_runtime::scene::{DefaultLevelManager, LevelMetadata, LevelSystem, NodeId, Scene};
use zircon_runtime_interface::math::{Mat4, Quat, Transform, UVec2, Vec2, Vec3};

const DOCUMENT: DocumentId = DocumentId::new(519);
const HISTORY: HistoryContextId = HistoryContextId::Document(DOCUMENT);
const PRESS: Vec2 = Vec2::new(400.0, 300.0);
const DRAG: Vec2 = Vec2::new(452.0, 300.0);
const SIGNED: Vec3 = Vec3::new(-2.0, -3.0, 0.5);
const ROOT_T: Vec3 = Vec3::new(4.0, -3.0, 1.0);
const CHILD_T: Vec3 = Vec3::new(2.0, 1.0, 3.0);

#[derive(Clone, Copy, Debug, PartialEq)]
struct Row {
    local: Transform,
    world: Mat4,
}

struct Drag {
    level: LevelSystem,
    engine: EditorTransactionEngine,
    controller: SceneViewportController,
    camera: ViewportCameraSnapshot,
    session: InteractiveTransformSession,
    pivot: Transform,
}

fn spawn(scene: &mut Scene, translation: Vec3, rotation: Quat, scale: Vec3) -> NodeId {
    let entity = scene.spawn_node(NodeKind::Empty).unwrap();
    scene
        .update_transform(
            entity,
            Transform {
                translation,
                rotation,
                scale,
            },
        )
        .unwrap();
    entity
}

impl Drag {
    fn begin(mut scene: Scene, selected: &[NodeId], primary: NodeId, mode: PivotMode) -> Self {
        scene.prepare_viewport_render_extract();
        let (root, pivot) =
            selection_pivot_transform(&scene, selected.iter().copied(), primary, mode).unwrap();
        let viewport = UVec2::new(800, 600);
        let mut camera = ViewportCameraSnapshot {
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 30.0)),
            ..ViewportCameraSnapshot::default()
        };
        camera.apply_viewport_size(viewport);
        let mut controller = SceneViewportController::new(viewport);
        controller
            .activate_scene_mode(SceneModeActivation::Transform(TransformHandleKind::Move))
            .unwrap();
        controller
            .apply_command(
                None,
                &ViewportCommand::SetTransformSpace(TransformSpace::Global),
            )
            .unwrap();
        assert!(controller.begin_handle_drag_for_transform(
            Some((root, pivot)),
            &camera,
            PRESS,
            GizmoAxis::X,
        ));
        let spec = controller.active_interactive_transform_spec().unwrap();
        assert_eq!(spec.kind(), InteractiveTransformKind::Move);

        let level = DefaultLevelManager::default().create_level(scene, LevelMetadata::default());
        let mut context = CoreEditContext::new(EditorRuntimeGatewayHandle::detached());
        context
            .bind_scene(
                level.clone(),
                SceneSelection::new(selected.to_vec(), Some(primary)),
            )
            .unwrap();
        let engine = EditorTransactionEngine::new(context);
        let (model, budget) = engine.retention_configuration_v1();
        let (retained, peak) = level
            .with_world(|scene| {
                InteractiveTransformSession::retention_projection_v1(
                    scene,
                    selected,
                    model,
                    budget.record_bytes,
                )
            })
            .unwrap();
        let reservation = engine
            .reserve_preview_retention_v1(HISTORY, retained, peak)
            .unwrap();
        let session = level
            .with_world(|scene| {
                InteractiveTransformSession::begin_reserved_v1(
                    scene,
                    selected,
                    primary,
                    spec,
                    mode,
                    DOCUMENT,
                    reservation,
                )
            })
            .unwrap();
        assert_eq!(session.primary_root(), root);
        assert_eq!(session.pivot_transform(), pivot);
        Self {
            level,
            engine,
            controller,
            camera,
            session,
            pivot,
        }
    }

    fn preview(&mut self, cursor: Vec2) -> Vec3 {
        let request = self
            .controller
            .update_handle_drag_for_transform(&self.camera, cursor)
            .expect("explicit Move preview, not merely a press/release");
        let target = request.target_pivot_world;
        assert_eq!(target.rotation, self.pivot.rotation);
        assert_eq!(target.scale, self.pivot.scale);
        let delta = target.translation - self.pivot.translation;
        assert_eq!(delta.y, 0.0);
        assert_eq!(delta.z, 0.0);
        self.level
            .with_world_mut(|world| {
                self.session
                    .preview(world, Some(DOCUMENT), request.primary, target)
            })
            .unwrap();
        delta
    }

    fn release(mut self) -> (LevelSystem, EditorTransactionEngine, Option<TransactionId>) {
        assert!(self.controller.finish_handle_drag_for_transform());
        let command = self
            .level
            .with_world(|world| self.session.finish(world, Some(DOCUMENT)))
            .unwrap();
        let id = command.map(|command| {
            let mut txn = self
                .engine
                .begin_with_retention_v1(
                    self.session.spec().kind().history_label(),
                    HISTORY,
                    self.session.retention_owner_v1().unwrap(),
                )
                .unwrap();
            txn.set_merge_mode(MergeMode::Disable);
            txn.push(EditorCommand::applied_transform_batch(command))
                .unwrap();
            txn.commit_after_apply(|_| Ok(())).unwrap()
        });
        (self.level, self.engine, id)
    }
}

fn published(level: &LevelSystem, nodes: &[NodeId]) -> (u64, Vec<Row>) {
    level.with_world_mut(|scene| {
        let revision = scene.world_generation();
        scene.prepare_viewport_render_extract();
        assert_eq!(
            scene.world_generation(),
            revision,
            "publishing is not authoring"
        );
        let rows = nodes
            .iter()
            .map(|&id| Row {
                local: scene.get::<LocalTransform>(id).unwrap().transform,
                world: scene
                    .get::<WorldMatrix>(id)
                    .expect("published Scene matrix")
                    .0,
            })
            .collect();
        (revision, rows)
    })
}

fn close(a: Vec3, b: Vec3) {
    assert!(
        (a - b).abs().max_element() <= 0.0001,
        "expected {b:?}, got {a:?}"
    );
}

fn linear(m: Mat4) -> [Vec3; 3] {
    [m.x_axis, m.y_axis, m.z_axis].map(|v| v.truncate())
}

fn same_world_basis(a: Mat4, b: Mat4) {
    for (x, y) in linear(a).into_iter().zip(linear(b)) {
        close(x, y);
    }
    assert_eq!(a.row(3), b.row(3));
}

fn kept_local_rs(a: Transform, b: Transform) {
    assert_eq!(
        a.scale.to_array().map(f32::to_bits),
        b.scale.to_array().map(f32::to_bits),
        "Move must not re-author signed local scale"
    );
    assert_eq!(
        a.rotation.to_array().map(f32::to_bits),
        b.rotation.to_array().map(f32::to_bits),
        "Move must not re-author local rotation"
    );
}

fn observe(label: &str, rows: &[Row], before: u64, after: u64, history: HistoryStatus) {
    for (index, row) in rows.iter().enumerate() {
        eprintln!("EDITOR_MOVE label={label} index={index} history={} dirty={} can_undo={} rev_before={before} rev_after={after} local={:?} rotation={:?} signed_scale={:?} origin={:?}",
            history.len, history.dirty, history.can_undo, row.local.translation.to_array(),
            row.local.rotation.to_array(), row.local.scale.to_array(), row.world.w_axis.truncate().to_array());
    }
}

fn signed_root_drag() -> (NodeId, Drag) {
    let mut scene = Scene::empty();
    let id = spawn(&mut scene, ROOT_T, Quat::IDENTITY, SIGNED);
    (id, Drag::begin(scene, &[id], id, PivotMode::Primary))
}

#[test]
fn same_cursor_move_preview_does_not_author_or_commit_history() {
    let (entity, mut drag) = signed_root_drag();
    let (rev, before) = published(&drag.level, &[entity]);
    let history_before = drag.engine.history_status(HISTORY).unwrap();
    assert_eq!(drag.preview(PRESS), Vec3::ZERO);
    let (level, engine, id) = drag.release();
    let (rev_after, after) = published(&level, &[entity]);
    let status = engine.history_status(HISTORY).unwrap();
    observe("same", &after, rev, rev_after, status);
    same_world_basis(after[0].world, before[0].world);
    close(after[0].world.w_axis.truncate(), ROOT_T);
    assert!(
        id.is_none(),
        "unchanged Move preview must not commit history"
    );
    assert_eq!(status, history_before);
    assert_eq!(rev_after, rev);
    kept_local_rs(after[0].local, before[0].local);
}

#[test]
fn real_move_out_and_back_does_not_commit_history() {
    let (entity, mut drag) = signed_root_drag();
    let (rev, before) = published(&drag.level, &[entity]);
    let history_before = drag.engine.history_status(HISTORY).unwrap();
    let delta = drag.preview(DRAG);
    assert!(delta.x > 0.1, "outgoing preview must actually move");
    let (rev_out, out) = published(&drag.level, &[entity]);
    assert!(rev_out > rev);
    close(out[0].world.w_axis.truncate(), ROOT_T + delta);
    same_world_basis(out[0].world, before[0].world);
    assert_eq!(drag.preview(PRESS), Vec3::ZERO);
    let (level, engine, id) = drag.release();
    let (rev_after, after) = published(&level, &[entity]);
    let status = engine.history_status(HISTORY).unwrap();
    observe("out_back", &after, rev, rev_after, status);
    close(after[0].world.w_axis.truncate(), ROOT_T);
    same_world_basis(after[0].world, before[0].world);
    assert!(id.is_none(), "return-to-start Move must not commit history");
    assert_eq!(status, history_before);
    assert_eq!(after[0].local, before[0].local);
}

fn sheared_attachment(scene: &mut Scene, sign: f32) -> (NodeId, NodeId) {
    let grand = spawn(
        scene,
        Vec3::new(8.0, -6.0, 2.0),
        Quat::IDENTITY,
        Vec3::new(2.0 * sign, 1.0, 4.0),
    );
    let mid = spawn(
        scene,
        Vec3::new(1.0, 2.0, -1.0),
        Quat::from_rotation_z(std::f32::consts::FRAC_PI_4),
        Vec3::ONE,
    );
    let child = spawn(scene, CHILD_T, Quat::IDENTITY, SIGNED);
    assert!(scene.set_parent_checked(mid, Some(grand)).unwrap());
    assert!(scene.set_parent_checked(child, Some(mid)).unwrap());
    (mid, child)
}

#[test]
fn centroid_move_preserves_signed_shear_through_real_undo_redo() {
    let mut scene = Scene::empty();
    let root = spawn(&mut scene, ROOT_T, Quat::IDENTITY, SIGNED);
    let desc = spawn(&mut scene, Vec3::X, Quat::IDENTITY, Vec3::ONE);
    assert!(scene.set_parent_checked(desc, Some(root)).unwrap());
    let (positive, plus) = sheared_attachment(&mut scene, 1.0);
    let (mirrored, minus) = sheared_attachment(&mut scene, -1.0);
    let selected = [root, desc, plus, minus];
    let mut drag = Drag::begin(scene, &selected, desc, PivotMode::Centroid);
    assert_eq!(
        drag.session.target_entities().collect::<Vec<_>>(),
        [root, plus, minus]
    );
    let (_, parents) = published(&drag.level, &[positive, mirrored]);
    for (row, sign) in parents.iter().zip([1.0, -1.0]) {
        let [x, y, _] = linear(row.world);
        assert!(
            (x.dot(y) + 1.5).abs() < 1.0e-5,
            "the immediate parent itself must be sheared"
        );
        assert_eq!(row.world.determinant().signum(), sign);
    }
    let (rev, before) = published(&drag.level, &selected);
    let c = std::f32::consts::FRAC_1_SQRT_2;
    let origins = [
        ROOT_T,
        ROOT_T - 2.0 * Vec3::X,
        Vec3::new(8.0 + 2.0 * (1.0 + c), -4.0 + 3.0 * c, 10.0),
        Vec3::new(8.0 - 2.0 * (1.0 + c), -4.0 + 3.0 * c, 10.0),
    ];
    for (row, expected) in before.iter().zip(origins) {
        close(row.world.w_axis.truncate(), expected);
    }
    close(
        drag.pivot.translation,
        (origins[0] + origins[2] + origins[3]) / 3.0,
    );
    assert_eq!(drag.engine.history_status(HISTORY).unwrap().len, 0);
    let delta = drag.preview(DRAG);
    assert!(delta.x > 0.1);
    let (rev_moved, moved) = published(&drag.level, &selected);
    assert!(rev_moved > rev);
    assert_eq!(
        drag.engine.history_status(HISTORY).unwrap().len,
        0,
        "preview is not history"
    );
    let (level, engine, id) = drag.release();
    let id = id.expect("real delta creates a retained history record");
    let history = engine.history_status(HISTORY).unwrap();
    assert_eq!(history.len, 1);
    assert_eq!(history.top, Some(id));
    assert!(history.dirty && history.can_undo && !history.can_redo);
    assert_eq!(published(&level, &selected).1, moved);
    observe("batch_move", &moved, rev, rev_moved, history);

    let journal = engine.journal_transaction(HISTORY, id).unwrap();
    assert_eq!(journal.commands().len(), 1);
    let command = &journal.commands()[0];
    assert_eq!(
        command.command_type(),
        "zircon.editor.scene.batch_transform"
    );
    let targets = command.payload()["targets"].as_array().unwrap();
    assert_eq!(targets.len(), 3);
    for (value, entity) in targets.iter().zip([root, plus, minus]) {
        assert_eq!(value["node_id"].as_u64(), Some(entity));
    }

    assert!(engine.undo(HISTORY).unwrap());
    let undo_status = engine.history_status(HISTORY).unwrap();
    assert!(!undo_status.can_undo && undo_status.can_redo);
    let (rev_undo, undo) = published(&level, &selected);
    observe("batch_undo", &undo, rev_moved, rev_undo, undo_status);
    assert_eq!(undo, before);
    assert!(engine.redo(HISTORY).unwrap());
    let redo_status = engine.history_status(HISTORY).unwrap();
    assert_eq!(redo_status.top, Some(id));
    let (rev_redo, redo) = published(&level, &selected);
    observe("batch_redo", &redo, rev_undo, rev_redo, redo_status);
    assert_eq!(redo, moved);
    assert_eq!(published(&level, &[positive, mirrored]).1, parents);

    // Independent Rz(-45) * D^-1 for the real global X delta.
    let x = delta.x * c / 2.0;
    for (index, local) in [
        ROOT_T + delta,
        Vec3::X,
        CHILD_T + Vec3::new(x, -x, 0.0),
        CHILD_T + Vec3::new(-x, x, 0.0),
    ]
    .into_iter()
    .enumerate()
    {
        close(redo[index].local.translation, local);
        close(redo[index].world.w_axis.truncate(), origins[index] + delta);
        same_world_basis(redo[index].world, before[index].world);
        kept_local_rs(redo[index].local, before[index].local);
    }
}
