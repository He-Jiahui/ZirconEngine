//! Editor host UI built on iced, with viewport frames coming from core graphics.

use iced::mouse;
use iced::task::Task;
use iced::widget::{
    button, column, container, image, mouse_area, row, scrollable, text, text_input,
};
use iced::{window, Element, Fill, Length, Point, Subscription, Theme};
use std::path::PathBuf;
use std::time::Duration;
use zircon_asset::{AssetWorkerPool, MeshSource};
use zircon_graphics::{
    EditorOrRuntimeFrame, GizmoAxis, RenderService, ViewportController, ViewportFeedback,
    ViewportFrame, ViewportInput, ViewportState,
};
use zircon_math::{UVec2, Vec2};
use zircon_scene::{NodeId, NodeKind, Scene};

const DEFAULT_PROJECT_PATH: &str = "sandbox.zircon-project.json";
const HISTORY_LIMIT: usize = 128;

#[derive(Clone, Debug)]
pub enum EditorIntent {
    CreateNode(NodeKind),
    SelectNode(NodeId),
    SetTransform(NodeId, zircon_math::Transform),
    BeginGizmoDrag,
    DragGizmo,
    EndGizmoDrag,
    Undo,
    Redo,
}

#[derive(Clone, Debug)]
pub struct SceneEntry {
    pub id: NodeId,
    pub name: String,
    pub selected: bool,
}

#[derive(Clone, Debug)]
pub struct InspectorSnapshot {
    pub id: NodeId,
    pub name: String,
    pub translation: [String; 3],
}

#[derive(Clone, Debug)]
pub struct EditorChromeSnapshot {
    pub scene_entries: Vec<SceneEntry>,
    pub inspector: Option<InspectorSnapshot>,
    pub status_line: String,
    pub hovered_axis: Option<GizmoAxis>,
    pub viewport_size: UVec2,
    pub mesh_import_path: String,
    pub project_path: String,
    pub can_undo: bool,
    pub can_redo: bool,
}

pub struct EditorPresenter;

#[derive(Clone, Debug, Default)]
struct EditorHistory {
    undo_stack: Vec<Scene>,
    redo_stack: Vec<Scene>,
    drag_origin: Option<Scene>,
}

impl EditorHistory {
    fn record(&mut self, scene: Scene) {
        self.undo_stack.push(scene);
        if self.undo_stack.len() > HISTORY_LIMIT {
            self.undo_stack.remove(0);
        }
        self.redo_stack.clear();
    }

    fn begin_drag(&mut self, scene: &Scene) {
        if self.drag_origin.is_none() {
            self.drag_origin = Some(scene.clone());
        }
    }

    fn end_drag(&mut self, scene: &Scene) -> bool {
        let Some(origin) = self.drag_origin.take() else {
            return false;
        };
        if origin != *scene {
            self.record(origin);
            true
        } else {
            false
        }
    }

    fn undo(&mut self, scene: &mut Scene) -> bool {
        self.drag_origin = None;
        let Some(previous) = self.undo_stack.pop() else {
            return false;
        };
        self.redo_stack.push(scene.clone());
        *scene = previous;
        true
    }

    fn redo(&mut self, scene: &mut Scene) -> bool {
        self.drag_origin = None;
        let Some(next) = self.redo_stack.pop() else {
            return false;
        };
        self.undo_stack.push(scene.clone());
        *scene = next;
        true
    }

    fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.drag_origin = None;
    }

    fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }
}

#[derive(Debug)]
pub struct EditorState {
    pub scene: Scene,
    viewport_controller: ViewportController,
    transform_fields: [String; 3],
    mesh_import_path: String,
    project_path: String,
    status_line: String,
    history: EditorHistory,
}

impl EditorState {
    pub fn new(viewport_size: UVec2) -> Self {
        let mut scene = Scene::new();
        let cube = scene
            .nodes()
            .iter()
            .find(|node| matches!(&node.kind, NodeKind::Cube))
            .map(|node| node.id)
            .unwrap_or(scene.active_camera());
        scene.set_selected(Some(cube));

        let mut state = Self {
            scene,
            viewport_controller: ViewportController::new(ViewportState::new(viewport_size)),
            transform_fields: Default::default(),
            mesh_import_path: String::new(),
            project_path: DEFAULT_PROJECT_PATH.to_string(),
            status_line: "Ready".to_string(),
            history: EditorHistory::default(),
        };
        state.sync_selection_state();
        state
    }

    pub fn apply_intent(&mut self, intent: EditorIntent) -> Result<bool, String> {
        match intent {
            EditorIntent::CreateNode(kind) => {
                let previous = self.scene.clone();
                let id = self.scene.spawn_node(kind);
                self.scene.set_selected(Some(id));
                self.commit_scene_change(previous);
                self.status_line = format!("Created node {id}");
                Ok(true)
            }
            EditorIntent::SelectNode(id) => {
                if self.scene.find_node(id).is_none() {
                    return Err(format!("Cannot select missing node {id}"));
                }
                self.scene.set_selected(Some(id));
                self.sync_selection_state();
                self.status_line = format!("Selected node {id}");
                Ok(true)
            }
            EditorIntent::SetTransform(id, transform) => {
                let Some(node) = self.scene.find_node(id).cloned() else {
                    return Err(format!("Cannot transform missing node {id}"));
                };
                if node.transform == transform {
                    return Ok(false);
                }
                let previous = self.scene.clone();
                self.scene.update_transform(id, transform);
                self.commit_scene_change(previous);
                self.status_line = format!("Updated transform for node {id}");
                Ok(true)
            }
            EditorIntent::BeginGizmoDrag => {
                self.history.begin_drag(&self.scene);
                self.status_line = "Translate gizmo drag".to_string();
                Ok(false)
            }
            EditorIntent::DragGizmo => {
                self.status_line = "Dragging translate gizmo".to_string();
                Ok(false)
            }
            EditorIntent::EndGizmoDrag => {
                let changed = self.history.end_drag(&self.scene);
                if changed {
                    self.sync_selection_state();
                }
                self.status_line = "Gizmo drag finished".to_string();
                Ok(false)
            }
            EditorIntent::Undo => {
                if self.history.undo(&mut self.scene) {
                    self.sync_selection_state();
                    self.status_line = "Undo".to_string();
                    Ok(true)
                } else {
                    self.status_line = "Nothing to undo".to_string();
                    Ok(false)
                }
            }
            EditorIntent::Redo => {
                if self.history.redo(&mut self.scene) {
                    self.sync_selection_state();
                    self.status_line = "Redo".to_string();
                    Ok(true)
                } else {
                    self.status_line = "Nothing to redo".to_string();
                    Ok(false)
                }
            }
        }
    }

    pub fn snapshot(&self) -> EditorChromeSnapshot {
        let inspector = self
            .scene
            .selected_node()
            .and_then(|id| self.scene.find_node(id).map(|node| (id, node)))
            .map(|(id, node)| InspectorSnapshot {
                id,
                name: node.name.clone(),
                translation: self.transform_fields.clone(),
            });

        EditorChromeSnapshot {
            scene_entries: self
                .scene
                .nodes()
                .iter()
                .map(|node| SceneEntry {
                    id: node.id,
                    name: node.name.clone(),
                    selected: self.scene.selected_node() == Some(node.id),
                })
                .collect(),
            inspector,
            status_line: self.status_line.clone(),
            hovered_axis: self.viewport_controller.hovered_axis(),
            viewport_size: self.viewport_controller.viewport().size,
            mesh_import_path: self.mesh_import_path.clone(),
            project_path: self.project_path.clone(),
            can_undo: self.history.can_undo(),
            can_redo: self.history.can_redo(),
        }
    }

    pub fn viewport_state(&self) -> ViewportState {
        self.viewport_controller.viewport().clone()
    }

    pub fn handle_viewport_input(&mut self, input: ViewportInput) -> ViewportFeedback {
        let feedback = self
            .viewport_controller
            .handle_input(&mut self.scene, input);
        if feedback.transformed_node.is_some() {
            self.sync_selection_state();
        }
        if let Some(selected) = self.scene.selected_node() {
            if let Some(node) = self.scene.find_node(selected) {
                self.viewport_controller
                    .set_orbit_target(node.transform.translation);
            }
        }
        if let Some(axis) = feedback.hovered_axis {
            self.status_line = format!("Hover gizmo axis {:?}", axis);
        }
        feedback
    }

    pub fn update_translation_field(&mut self, axis: usize, value: String) -> bool {
        self.transform_fields[axis] = value;
        let Some(node_id) = self.scene.selected_node() else {
            return false;
        };
        let Some(node) = self.scene.find_node(node_id).cloned() else {
            return false;
        };
        let parsed = [
            self.transform_fields[0].parse::<f32>(),
            self.transform_fields[1].parse::<f32>(),
            self.transform_fields[2].parse::<f32>(),
        ];
        if let [Ok(x), Ok(y), Ok(z)] = parsed {
            let transform = zircon_math::Transform {
                translation: zircon_math::Vec3::new(x, y, z),
                ..node.transform
            };
            return self
                .apply_intent(EditorIntent::SetTransform(node_id, transform))
                .unwrap_or_else(|error| {
                    self.status_line = error;
                    false
                });
        }
        false
    }

    pub fn set_mesh_import_path(&mut self, value: String) {
        self.mesh_import_path = value;
    }

    pub fn set_project_path(&mut self, value: String) {
        self.project_path = value;
    }

    pub fn import_mesh_from_path(&mut self) -> Result<bool, String> {
        let canonical = canonical_mesh_path(&self.mesh_import_path)?;
        let previous = self.scene.clone();
        let mesh = MeshSource::Path(canonical.to_string_lossy().into_owned());
        let id = self.scene.spawn_mesh_node(mesh);
        self.scene.set_selected(Some(id));
        self.mesh_import_path = canonical.to_string_lossy().into_owned();
        self.commit_scene_change(previous);
        self.status_line = format!("Imported mesh node {id}");
        Ok(true)
    }

    pub fn save_project(&mut self) -> Result<(), String> {
        let path = trimmed_path(&self.project_path, "project path")?;
        self.scene
            .save_project_to_path(&path)
            .map_err(|error| format!("Save project failed: {error}"))?;
        self.project_path = path.to_string_lossy().into_owned();
        self.status_line = format!("Saved project to {}", self.project_path);
        Ok(())
    }

    pub fn load_project(&mut self) -> Result<bool, String> {
        let path = trimmed_path(&self.project_path, "project path")?;
        let scene = Scene::load_project_from_path(&path)
            .map_err(|error| format!("Load project failed: {error}"))?;
        self.scene = scene;
        self.project_path = path.to_string_lossy().into_owned();
        self.history.clear();
        self.sync_selection_state();
        self.status_line = format!("Loaded project from {}", self.project_path);
        Ok(true)
    }

    pub fn set_status_line(&mut self, value: impl Into<String>) {
        self.status_line = value.into();
    }

    fn commit_scene_change(&mut self, previous: Scene) {
        self.history.record(previous);
        self.sync_selection_state();
    }

    fn sync_selection_state(&mut self) {
        if let Some(selected) = self.scene.selected_node() {
            if let Some(node) = self.scene.find_node(selected) {
                self.transform_fields = [
                    format!("{:.2}", node.transform.translation.x),
                    format!("{:.2}", node.transform.translation.y),
                    format!("{:.2}", node.transform.translation.z),
                ];
                self.viewport_controller
                    .set_orbit_target(node.transform.translation);
                return;
            }
        }

        self.transform_fields = [String::new(), String::new(), String::new()];
    }
}

struct EditorApp {
    state: EditorState,
    chrome: EditorChromeSnapshot,
    render_service: RenderService,
    _asset_workers: AssetWorkerPool,
    viewport_frame: Option<ViewportFrame>,
    viewport_image: Option<image::Handle>,
    viewport_cursor: Vec2,
    dragging_gizmo: bool,
    dirty: bool,
}

#[derive(Clone, Debug)]
enum Message {
    Tick,
    WindowResized(iced::Size),
    CreateNode(NodeKind),
    SelectNode(NodeId),
    TranslateXChanged(String),
    TranslateYChanged(String),
    TranslateZChanged(String),
    Undo,
    Redo,
    MeshImportPathChanged(String),
    ImportMesh,
    ProjectPathChanged(String),
    SaveProject,
    LoadProject,
    ViewportMoved(Point),
    ViewportLeftPress,
    ViewportLeftRelease,
    ViewportRightPress,
    ViewportRightRelease,
    ViewportMiddlePress,
    ViewportMiddleRelease,
    ViewportScroll(mouse::ScrollDelta),
}

impl EditorApp {
    fn boot() -> (Self, Task<Message>) {
        let window_size = iced::Size::new(1440.0, 900.0);
        let viewport_size = estimate_viewport_size(window_size);
        let asset_workers = AssetWorkerPool::new(
            std::thread::available_parallelism().map_or(2, |n| n.get().max(2) - 1),
        )
        .expect("asset workers");
        let render_service = RenderService::spawn(
            asset_workers.request_sender(),
            asset_workers.completion_receiver(),
        )
        .unwrap_or_else(|error| panic!("render service bootstrap failed: {error}"));

        let mut state = EditorState::new(viewport_size);
        state.status_line = "Editor booted".to_string();
        let chrome = state.snapshot();

        (
            Self {
                state,
                chrome,
                render_service,
                _asset_workers: asset_workers,
                viewport_frame: None,
                viewport_image: None,
                viewport_cursor: Vec2::ZERO,
                dragging_gizmo: false,
                dirty: true,
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => {
                if self.dirty {
                    let _ = self.render_service.submit_frame(EditorOrRuntimeFrame {
                        scene: self.state.scene.to_render_snapshot(),
                        viewport: self.state.viewport_state(),
                    });
                    self.dirty = false;
                }
                if let Some(frame) = self.render_service.try_recv_latest_frame() {
                    self.viewport_image = Some(image::Handle::from_rgba(
                        frame.width,
                        frame.height,
                        frame.rgba.clone(),
                    ));
                    self.viewport_frame = Some(frame);
                }
            }
            Message::WindowResized(size) => {
                let viewport = estimate_viewport_size(size);
                self.state
                    .handle_viewport_input(ViewportInput::Resized(viewport));
                self.dirty = true;
            }
            Message::CreateNode(kind) => {
                let result = self.state.apply_intent(EditorIntent::CreateNode(kind));
                self.apply_result(result);
            }
            Message::SelectNode(id) => {
                let result = self.state.apply_intent(EditorIntent::SelectNode(id));
                self.apply_result(result);
            }
            Message::TranslateXChanged(value) => {
                self.dirty |= self.state.update_translation_field(0, value);
            }
            Message::TranslateYChanged(value) => {
                self.dirty |= self.state.update_translation_field(1, value);
            }
            Message::TranslateZChanged(value) => {
                self.dirty |= self.state.update_translation_field(2, value);
            }
            Message::Undo => {
                let result = self.state.apply_intent(EditorIntent::Undo);
                self.apply_result(result);
            }
            Message::Redo => {
                let result = self.state.apply_intent(EditorIntent::Redo);
                self.apply_result(result);
            }
            Message::MeshImportPathChanged(value) => {
                self.state.set_mesh_import_path(value);
            }
            Message::ImportMesh => {
                let result = self.state.import_mesh_from_path();
                self.apply_result(result);
            }
            Message::ProjectPathChanged(value) => {
                self.state.set_project_path(value);
            }
            Message::SaveProject => {
                if let Err(error) = self.state.save_project() {
                    self.state.set_status_line(error);
                }
            }
            Message::LoadProject => {
                let result = self.state.load_project();
                self.apply_result(result);
            }
            Message::ViewportMoved(position) => {
                self.viewport_cursor = Vec2::new(position.x, position.y);
                let feedback = self
                    .state
                    .handle_viewport_input(ViewportInput::PointerMoved(self.viewport_cursor));
                if self.dragging_gizmo && feedback.transformed_node.is_some() {
                    let _ = self.state.apply_intent(EditorIntent::DragGizmo);
                }
                self.dirty |= feedback.camera_updated
                    || feedback.transformed_node.is_some()
                    || feedback.hovered_axis.is_some();
            }
            Message::ViewportLeftPress => {
                let feedback = self
                    .state
                    .handle_viewport_input(ViewportInput::LeftPressed(self.viewport_cursor));
                self.dragging_gizmo = feedback.hovered_axis.is_some();
                if self.dragging_gizmo {
                    let _ = self.state.apply_intent(EditorIntent::BeginGizmoDrag);
                }
                self.dirty |=
                    feedback.transformed_node.is_some() || feedback.hovered_axis.is_some();
            }
            Message::ViewportLeftRelease => {
                if self.dragging_gizmo {
                    let _ = self.state.apply_intent(EditorIntent::EndGizmoDrag);
                }
                self.dragging_gizmo = false;
                let _ = self
                    .state
                    .handle_viewport_input(ViewportInput::LeftReleased);
            }
            Message::ViewportRightPress => {
                let _ = self
                    .state
                    .handle_viewport_input(ViewportInput::RightPressed(self.viewport_cursor));
            }
            Message::ViewportRightRelease => {
                let _ = self
                    .state
                    .handle_viewport_input(ViewportInput::RightReleased);
            }
            Message::ViewportMiddlePress => {
                let _ = self
                    .state
                    .handle_viewport_input(ViewportInput::MiddlePressed(self.viewport_cursor));
            }
            Message::ViewportMiddleRelease => {
                let _ = self
                    .state
                    .handle_viewport_input(ViewportInput::MiddleReleased);
            }
            Message::ViewportScroll(delta) => {
                let amount = match delta {
                    mouse::ScrollDelta::Lines { y, .. } => y,
                    mouse::ScrollDelta::Pixels { y, .. } => y as f32 * 0.1,
                };
                let feedback = self
                    .state
                    .handle_viewport_input(ViewportInput::Scrolled(amount));
                self.dirty |= feedback.camera_updated;
            }
        }

        self.chrome = self.state.snapshot();
        Task::none()
    }

    fn apply_result(&mut self, result: Result<bool, String>) {
        match result {
            Ok(changed) => {
                self.dirty |= changed;
            }
            Err(error) => self.state.set_status_line(error),
        }
    }

    fn view(&self) -> Element<'_, Message> {
        EditorPresenter::compose(&self.chrome, self.viewport_image.clone())
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch(vec![
            iced::time::every(Duration::from_millis(16)).map(|_| Message::Tick),
            window::resize_events().map(|(_, size)| Message::WindowResized(size)),
        ])
    }
}

impl EditorPresenter {
    fn compose(
        chrome: &EditorChromeSnapshot,
        viewport_image: Option<image::Handle>,
    ) -> Element<'_, Message> {
        let add_row = row![
            button("Add Cube").on_press(Message::CreateNode(NodeKind::Cube)),
            button("Add Camera").on_press(Message::CreateNode(NodeKind::Camera)),
            button("Add Light").on_press(Message::CreateNode(NodeKind::DirectionalLight)),
            maybe_enabled_button("Undo", chrome.can_undo, Message::Undo),
            maybe_enabled_button("Redo", chrome.can_redo, Message::Redo),
        ]
        .spacing(8);

        let import_row = row![
            text_input("Mesh .obj path", &chrome.mesh_import_path)
                .on_input(Message::MeshImportPathChanged)
                .width(Length::Fixed(260.0)),
            maybe_enabled_button(
                "Import Mesh",
                !chrome.mesh_import_path.trim().is_empty(),
                Message::ImportMesh,
            ),
            text_input("Project path", &chrome.project_path)
                .on_input(Message::ProjectPathChanged)
                .width(Length::Fixed(260.0)),
            maybe_enabled_button(
                "Open Project",
                !chrome.project_path.trim().is_empty(),
                Message::LoadProject,
            ),
            maybe_enabled_button(
                "Save Project",
                !chrome.project_path.trim().is_empty(),
                Message::SaveProject,
            ),
        ]
        .spacing(8);

        let toolbar = column![add_row, import_row].spacing(8);

        let scene_tree =
            chrome
                .scene_entries
                .iter()
                .fold(column![text("Scene").size(20)], |column, entry| {
                    let label = if entry.selected {
                        format!("> {}", entry.name)
                    } else {
                        entry.name.clone()
                    };
                    column.push(
                        button(text(label))
                            .width(Fill)
                            .on_press(Message::SelectNode(entry.id)),
                    )
                });

        let inspector = if let Some(inspector) = chrome.inspector.as_ref() {
            column![
                text("Inspector").size(20),
                text(format!("{} ({})", inspector.name, inspector.id)),
                text_input("X", &inspector.translation[0]).on_input(Message::TranslateXChanged),
                text_input("Y", &inspector.translation[1]).on_input(Message::TranslateYChanged),
                text_input("Z", &inspector.translation[2]).on_input(Message::TranslateZChanged),
                text(format!(
                    "Hovered Axis: {}",
                    hovered_axis_label(chrome.hovered_axis)
                )),
            ]
            .spacing(8)
        } else {
            column![text("Inspector").size(20), text("Nothing selected")].spacing(8)
        };

        let viewport_content: Element<'_, Message> = if let Some(handle) = viewport_image {
            mouse_area(
                image(handle)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .content_fit(iced::ContentFit::Fill),
            )
            .on_move(Message::ViewportMoved)
            .on_press(Message::ViewportLeftPress)
            .on_release(Message::ViewportLeftRelease)
            .on_right_press(Message::ViewportRightPress)
            .on_right_release(Message::ViewportRightRelease)
            .on_middle_press(Message::ViewportMiddlePress)
            .on_middle_release(Message::ViewportMiddleRelease)
            .on_scroll(Message::ViewportScroll)
            .into()
        } else {
            container(text("Waiting for viewport frame..."))
                .center_x(Fill)
                .center_y(Fill)
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        };

        let viewport = column![
            text(format!(
                "Viewport {}x{}",
                chrome.viewport_size.x, chrome.viewport_size.y
            )),
            container(viewport_content)
                .width(Length::Fill)
                .height(Length::Fill)
        ]
        .spacing(8)
        .width(Length::Fill)
        .height(Length::Fill);

        let body = row![
            container(scrollable(scene_tree))
                .width(260)
                .height(Length::Fill),
            container(viewport).width(Length::Fill).height(Length::Fill),
            container(inspector).width(280).height(Length::Fill),
        ]
        .spacing(12)
        .height(Length::Fill);

        column![
            toolbar,
            body,
            container(text(chrome.status_line.clone())).width(Length::Fill),
        ]
        .spacing(12)
        .padding(12)
        .into()
    }
}

fn estimate_viewport_size(window_size: iced::Size) -> UVec2 {
    let width = (window_size.width - 24.0 - 260.0 - 280.0 - 24.0).max(1.0);
    let height = (window_size.height - 24.0 - 72.0 - 32.0 - 24.0).max(1.0);
    UVec2::new(width as u32, height as u32)
}

fn hovered_axis_label(axis: Option<GizmoAxis>) -> &'static str {
    match axis {
        Some(GizmoAxis::X) => "X",
        Some(GizmoAxis::Y) => "Y",
        Some(GizmoAxis::Z) => "Z",
        None => "None",
    }
}

fn maybe_enabled_button<'a>(
    label: &'a str,
    enabled: bool,
    message: Message,
) -> iced::widget::Button<'a, Message> {
    if enabled {
        button(label).on_press(message)
    } else {
        button(label)
    }
}

fn trimmed_path(value: &str, label: &str) -> Result<PathBuf, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(format!("{label} is empty"));
    }
    Ok(PathBuf::from(trimmed))
}

fn canonical_mesh_path(value: &str) -> Result<PathBuf, String> {
    let path = trimmed_path(value, "mesh import path")?;
    let extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    if extension != "obj" {
        return Err("Only .obj mesh import is supported right now".to_string());
    }
    let canonical = path
        .canonicalize()
        .map_err(|error| format!("Cannot access mesh {}: {error}", path.display()))?;
    if !canonical.is_file() {
        return Err(format!("Mesh path is not a file: {}", canonical.display()));
    }
    Ok(canonical)
}

fn editor_title(_: &EditorApp) -> String {
    "Zircon Editor".to_string()
}

fn editor_theme(_: &EditorApp) -> Theme {
    Theme::TokyoNight
}

pub fn run_editor() -> iced::Result {
    iced::application(EditorApp::boot, EditorApp::update, EditorApp::view)
        .subscription(EditorApp::subscription)
        .title(editor_title)
        .theme(editor_theme)
        .window_size(iced::Size::new(1440.0, 900.0))
        .resizable(true)
        .run()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn undo_redo_restores_created_nodes() {
        let mut state = EditorState::new(UVec2::new(1280, 720));
        let initial_count = state.scene.nodes().len();

        assert!(state
            .apply_intent(EditorIntent::CreateNode(NodeKind::Cube))
            .unwrap());
        assert_eq!(state.scene.nodes().len(), initial_count + 1);

        assert!(state.apply_intent(EditorIntent::Undo).unwrap());
        assert_eq!(state.scene.nodes().len(), initial_count);

        assert!(state.apply_intent(EditorIntent::Redo).unwrap());
        assert_eq!(state.scene.nodes().len(), initial_count + 1);
    }

    #[test]
    fn imported_mesh_can_be_undone() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let mesh_path = std::env::temp_dir().join(format!("zircon_editor_mesh_{unique}.obj"));
        fs::write(
            &mesh_path,
            "\
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.0 1.0 0.0
f 1 2 3
",
        )
        .unwrap();

        let mut state = EditorState::new(UVec2::new(1280, 720));
        let initial_count = state.scene.nodes().len();
        state.set_mesh_import_path(mesh_path.to_string_lossy().into_owned());

        assert!(state.import_mesh_from_path().unwrap());
        assert_eq!(state.scene.nodes().len(), initial_count + 1);
        assert!(matches!(
            state.scene.nodes().last().map(|node| &node.kind),
            Some(NodeKind::Mesh)
        ));

        assert!(state.apply_intent(EditorIntent::Undo).unwrap());
        assert_eq!(state.scene.nodes().len(), initial_count);

        let _ = fs::remove_file(mesh_path);
    }
}
