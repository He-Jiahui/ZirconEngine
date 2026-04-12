//! Minimal scene graph, project persistence, and render snapshot conversion.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use thiserror::Error;
use zircon_asset::{MeshSource, TextureSource};
use zircon_math::{Quat, Transform, Vec3, Vec4};

pub type NodeId = u64;
const PROJECT_FORMAT_VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeKind {
    Camera,
    Cube,
    Mesh,
    DirectionalLight,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CameraComponent {
    pub fov_y_radians: f32,
    pub z_near: f32,
    pub z_far: f32,
}

impl Default for CameraComponent {
    fn default() -> Self {
        Self {
            fov_y_radians: 60.0_f32.to_radians(),
            z_near: 0.1,
            z_far: 200.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MeshComponent {
    pub mesh: MeshSource,
    pub texture: TextureSource,
    pub tint: Vec4,
}

impl MeshComponent {
    pub fn from_source(mesh: MeshSource) -> Self {
        Self {
            mesh,
            texture: TextureSource::BuiltinChecker,
            tint: Vec4::ONE,
        }
    }
}

impl Default for MeshComponent {
    fn default() -> Self {
        Self::from_source(MeshSource::BuiltinCube)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DirectionalLightComponent {
    pub direction: Vec3,
    pub color: Vec3,
    pub intensity: f32,
}

impl Default for DirectionalLightComponent {
    fn default() -> Self {
        Self {
            direction: Vec3::new(-0.4, -1.0, -0.25).normalize_or_zero(),
            color: Vec3::splat(1.0),
            intensity: 2.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneNode {
    pub id: NodeId,
    pub name: String,
    pub kind: NodeKind,
    pub parent: Option<NodeId>,
    pub transform: Transform,
    pub camera: Option<CameraComponent>,
    pub mesh: Option<MeshComponent>,
    pub directional_light: Option<DirectionalLightComponent>,
}

impl SceneNode {
    fn new(id: NodeId, name: String, kind: NodeKind) -> Self {
        let mut node = Self {
            id,
            name,
            kind,
            parent: None,
            transform: Transform::default(),
            camera: None,
            mesh: None,
            directional_light: None,
        };

        match node.kind {
            NodeKind::Camera => {
                node.camera = Some(CameraComponent::default());
                node.transform =
                    Transform::looking_at(Vec3::new(3.0, 2.0, 5.0), Vec3::ZERO, Vec3::Y);
            }
            NodeKind::Cube => {
                node.mesh = Some(MeshComponent::default());
            }
            NodeKind::Mesh => {
                node.mesh = Some(MeshComponent::from_source(MeshSource::BuiltinCube));
            }
            NodeKind::DirectionalLight => {
                node.directional_light = Some(DirectionalLightComponent::default());
                node.transform.rotation = Quat::from_rotation_x(-45.0_f32.to_radians());
            }
        }

        node
    }

    fn new_mesh_instance(id: NodeId, name: String, source: MeshSource) -> Self {
        let mut node = Self::new(id, name, NodeKind::Mesh);
        node.mesh = Some(MeshComponent::from_source(source));
        node
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RenderCameraSnapshot {
    pub node_id: NodeId,
    pub transform: Transform,
    pub fov_y_radians: f32,
    pub z_near: f32,
    pub z_far: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RenderMeshSnapshot {
    pub node_id: NodeId,
    pub transform: Transform,
    pub mesh: MeshSource,
    pub texture: TextureSource,
    pub tint: Vec4,
    pub selected: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RenderDirectionalLightSnapshot {
    pub direction: Vec3,
    pub color: Vec3,
    pub intensity: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RenderGizmoSnapshot {
    pub target_node: NodeId,
    pub origin: Vec3,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RenderSceneSnapshot {
    pub camera: RenderCameraSnapshot,
    pub meshes: Vec<RenderMeshSnapshot>,
    pub light: RenderDirectionalLightSnapshot,
    pub selected_node: Option<NodeId>,
    pub gizmo: Option<RenderGizmoSnapshot>,
    pub show_grid: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Scene {
    nodes: Vec<SceneNode>,
    next_id: NodeId,
    active_camera: NodeId,
    selected_node: Option<NodeId>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct ProjectDocument {
    format_version: u32,
    scene: Scene,
}

#[derive(Debug, Error)]
pub enum SceneProjectError {
    #[error("project I/O failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("project parse failed: {0}")]
    Parse(#[from] serde_json::Error),
}

impl Scene {
    pub fn new() -> Self {
        let mut scene = Self {
            nodes: Vec::new(),
            next_id: 1,
            active_camera: 0,
            selected_node: None,
        };

        let camera_id = scene.spawn_node(NodeKind::Camera);
        scene.active_camera = camera_id;
        scene.spawn_node(NodeKind::DirectionalLight);
        scene.spawn_node(NodeKind::Cube);
        scene.set_selected(Some(camera_id));
        scene
    }

    pub fn spawn_node(&mut self, kind: NodeKind) -> NodeId {
        let id = self.next_id;
        self.next_id += 1;
        let ordinal = self.nodes.iter().filter(|node| node.kind == kind).count() + 1;
        let name = match kind {
            NodeKind::Camera => format!("Camera {ordinal}"),
            NodeKind::Cube => format!("Cube {ordinal}"),
            NodeKind::Mesh => format!("Mesh {ordinal}"),
            NodeKind::DirectionalLight => format!("Directional Light {ordinal}"),
        };

        let node = SceneNode::new(id, name, kind.clone());
        self.nodes.push(node);

        if matches!(kind, NodeKind::Camera) && self.active_camera == 0 {
            self.active_camera = id;
        }

        id
    }

    pub fn spawn_mesh_node(&mut self, mesh: MeshSource) -> NodeId {
        let id = self.next_id;
        self.next_id += 1;
        let name = mesh_display_name(&mesh, self.nodes.len() + 1);
        let node = SceneNode::new_mesh_instance(id, name, mesh);
        self.nodes.push(node);
        id
    }

    pub fn active_camera(&self) -> NodeId {
        self.active_camera
    }

    pub fn set_active_camera(&mut self, node_id: NodeId) {
        if self
            .find_node(node_id)
            .is_some_and(|node| matches!(node.kind, NodeKind::Camera))
        {
            self.active_camera = node_id;
        }
    }

    pub fn set_selected(&mut self, node_id: Option<NodeId>) {
        self.selected_node = node_id;
    }

    pub fn selected_node(&self) -> Option<NodeId> {
        self.selected_node
    }

    pub fn nodes(&self) -> &[SceneNode] {
        &self.nodes
    }

    pub fn find_node(&self, node_id: NodeId) -> Option<&SceneNode> {
        self.nodes.iter().find(|node| node.id == node_id)
    }

    pub fn find_node_mut(&mut self, node_id: NodeId) -> Option<&mut SceneNode> {
        self.nodes.iter_mut().find(|node| node.id == node_id)
    }

    pub fn update_transform(&mut self, node_id: NodeId, transform: Transform) {
        if let Some(node) = self.find_node_mut(node_id) {
            node.transform = transform;
        }
    }

    pub fn save_project_to_path(&self, path: impl AsRef<Path>) -> Result<(), SceneProjectError> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }
        let document = ProjectDocument {
            format_version: PROJECT_FORMAT_VERSION,
            scene: self.clone(),
        };
        let json = serde_json::to_string_pretty(&document)?;
        fs::write(path, json)?;
        Ok(())
    }

    pub fn load_project_from_path(path: impl AsRef<Path>) -> Result<Self, SceneProjectError> {
        let json = fs::read_to_string(path)?;
        let mut document: ProjectDocument = serde_json::from_str(&json)?;
        document.scene.normalize_after_load();
        Ok(document.scene)
    }

    pub fn to_render_snapshot(&self) -> RenderSceneSnapshot {
        let camera_node = self
            .find_node(self.active_camera)
            .and_then(|node| node.camera.as_ref().map(|camera| (node, camera)))
            .unwrap_or_else(|| {
                self.nodes
                    .iter()
                    .find_map(|node| node.camera.as_ref().map(|camera| (node, camera)))
                    .expect("scene always contains a camera")
            });
        let light = self
            .nodes
            .iter()
            .find_map(|node| node.directional_light.as_ref())
            .cloned()
            .unwrap_or_default();

        let meshes = self
            .nodes
            .iter()
            .filter_map(|node| {
                node.mesh.as_ref().map(|mesh| RenderMeshSnapshot {
                    node_id: node.id,
                    transform: node.transform,
                    mesh: mesh.mesh.clone(),
                    texture: mesh.texture.clone(),
                    tint: mesh.tint,
                    selected: self.selected_node == Some(node.id),
                })
            })
            .collect();

        let gizmo = self
            .selected_node
            .and_then(|node_id| self.find_node(node_id))
            .filter(|node| !matches!(node.kind, NodeKind::DirectionalLight))
            .map(|node| RenderGizmoSnapshot {
                target_node: node.id,
                origin: node.transform.translation,
            });

        RenderSceneSnapshot {
            camera: RenderCameraSnapshot {
                node_id: camera_node.0.id,
                transform: camera_node.0.transform,
                fov_y_radians: camera_node.1.fov_y_radians,
                z_near: camera_node.1.z_near,
                z_far: camera_node.1.z_far,
            },
            meshes,
            light: RenderDirectionalLightSnapshot {
                direction: light.direction,
                color: light.color,
                intensity: light.intensity,
            },
            selected_node: self.selected_node,
            gizmo,
            show_grid: true,
        }
    }

    fn normalize_after_load(&mut self) {
        self.next_id = self.nodes.iter().map(|node| node.id).max().unwrap_or(0) + 1;
        if self
            .nodes
            .iter()
            .all(|node| !matches!(node.kind, NodeKind::Camera))
        {
            self.active_camera = self.spawn_node(NodeKind::Camera);
        } else if self.find_node(self.active_camera).is_none()
            || !matches!(
                self.find_node(self.active_camera).map(|node| &node.kind),
                Some(NodeKind::Camera)
            )
        {
            self.active_camera = self
                .nodes
                .iter()
                .find(|node| matches!(node.kind, NodeKind::Camera))
                .map(|node| node.id)
                .expect("normalize ensures a camera exists");
        }

        if self
            .nodes
            .iter()
            .all(|node| !matches!(node.kind, NodeKind::DirectionalLight))
        {
            self.spawn_node(NodeKind::DirectionalLight);
        }

        if self
            .selected_node
            .is_some_and(|id| self.find_node(id).is_none())
        {
            self.selected_node = None;
        }
        if self.selected_node.is_none() {
            self.selected_node = Some(self.active_camera);
        }
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}

fn mesh_display_name(mesh: &MeshSource, fallback_ordinal: usize) -> String {
    match mesh {
        MeshSource::BuiltinCube => format!("Cube {fallback_ordinal}"),
        MeshSource::Path(path) => Path::new(path)
            .file_stem()
            .and_then(|stem| stem.to_str())
            .filter(|stem| !stem.is_empty())
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| format!("Mesh {fallback_ordinal}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn scene_bootstraps_with_renderable_defaults() {
        let scene = Scene::new();
        let snapshot = scene.to_render_snapshot();

        assert!(!snapshot.meshes.is_empty());
        assert!(snapshot.show_grid);
    }

    #[test]
    fn spawn_node_returns_unique_ids() {
        let mut scene = Scene::new();
        let first = scene.spawn_node(NodeKind::Cube);
        let second = scene.spawn_node(NodeKind::Cube);

        assert_ne!(first, second);
    }

    #[test]
    fn updated_transform_is_reflected_in_render_snapshot() {
        let mut scene = Scene::new();
        let cube = scene
            .nodes()
            .iter()
            .find(|node| matches!(node.kind, NodeKind::Cube))
            .unwrap()
            .id;
        let mut transform = scene.find_node(cube).unwrap().transform;
        transform.translation = Vec3::new(2.0, 3.0, 4.0);

        scene.update_transform(cube, transform);

        let snapshot = scene.to_render_snapshot();
        let mesh_snapshot = snapshot
            .meshes
            .iter()
            .find(|mesh_snapshot| mesh_snapshot.node_id == cube)
            .unwrap();
        assert_eq!(
            mesh_snapshot.transform.translation,
            Vec3::new(2.0, 3.0, 4.0)
        );
    }

    #[test]
    fn project_roundtrip_preserves_imported_meshes() {
        let mut scene = Scene::new();
        let imported = scene.spawn_mesh_node(MeshSource::Path("E:/assets/robot.obj".to_string()));
        scene.set_selected(Some(imported));

        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("zircon_scene_roundtrip_{unique}.json"));
        scene.save_project_to_path(&path).unwrap();
        let loaded = Scene::load_project_from_path(&path).unwrap();
        let _ = fs::remove_file(&path);

        assert_eq!(loaded.selected_node(), Some(imported));
        let imported_node = loaded.find_node(imported).unwrap();
        assert!(matches!(imported_node.kind, NodeKind::Mesh));
        assert_eq!(
            imported_node.mesh.as_ref().unwrap().mesh,
            MeshSource::Path("E:/assets/robot.obj".to_string())
        );
    }
}
