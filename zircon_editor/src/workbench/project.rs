//! Directory-style project document helpers for editor workspace persistence.

use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use zircon_asset::{
    AlphaMode, AssetImportError, AssetReference, MaterialAsset, ProjectManager,
    ProjectManifest, ProjectPaths,
};
use zircon_resource::ResourceLocator;
use zircon_scene::{Scene, SceneProjectError};

use crate::layout::{ActivityDrawerSlot, WorkbenchLayout};
use crate::workbench::startup::NewProjectDraft;
use crate::view::{ViewInstance, ViewInstanceId};

const EDITOR_PROJECT_FORMAT_VERSION: u32 = 1;
const DEFAULT_SCENE_URI: &str = "res://scenes/main.scene.toml";
const DEFAULT_SHADER_URI: &str = "res://shaders/pbr.wgsl";
const EDITOR_WORKSPACE_DIR: &str = ".zircon";
const EDITOR_WORKSPACE_FILE: &str = "editor-workspace.json";
const EDITOR_LAYOUT_PRESET_FORMAT_VERSION: u32 = 1;
const EDITOR_LAYOUT_PRESET_DIR: &str = "editor/layout-presets";
const EDITOR_LAYOUT_PRESET_SUFFIX: &str = ".workbench-layout.json";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProjectEditorWorkspace {
    pub layout_version: u32,
    pub workbench: WorkbenchLayout,
    pub open_view_instances: Vec<ViewInstance>,
    pub active_center_tab: Option<ViewInstanceId>,
    pub active_drawers: Vec<ActivityDrawerSlot>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct EditorProjectDocument {
    pub root_path: PathBuf,
    pub manifest: ProjectManifest,
    pub world: Scene,
    pub editor_workspace: Option<ProjectEditorWorkspace>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct EditorWorkspaceDocument {
    format_version: u32,
    editor_workspace: ProjectEditorWorkspace,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct LayoutPresetAssetDocument {
    format_version: u32,
    preset_name: String,
    workbench: WorkbenchLayout,
}

impl EditorProjectDocument {
    pub fn create_renderable_template(
        draft: &NewProjectDraft,
    ) -> Result<PathBuf, SceneProjectError> {
        let root = draft
            .validate_for_creation()
            .map_err(|error| invalid_data(error))?;
        if !root.exists() {
            fs::create_dir_all(&root)?;
        }
        Self::ensure_runtime_assets(&root)?;

        let paths = ProjectPaths::from_root(&root)?;
        if !paths.manifest_path().exists() {
            let default_scene = parse_asset_uri(DEFAULT_SCENE_URI)?;
            ProjectManifest::new(&draft.project_name, default_scene, 1).save(paths.manifest_path())?;
        }

        let mut project = ProjectManager::open(&root)?;
        project.scan_and_import()?;
        let scene = Scene::default();
        let scene_asset = scene.to_scene_asset(&project)?;
        let scene_path = project.source_path_for_uri(&project.manifest().default_scene)?;
        if let Some(parent) = scene_path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }
        fs::write(
            scene_path,
            scene_asset
                .to_toml_string()
                .map_err(|error| invalid_data(error.to_string()))?,
        )?;
        Ok(root)
    }

    pub fn load_from_path(path: impl AsRef<Path>) -> Result<Self, SceneProjectError> {
        let root = project_root_path(path)?;
        let mut project = ProjectManager::open(&root)?;
        project.scan_and_import()?;

        Ok(Self {
            root_path: root.clone(),
            manifest: project.manifest().clone(),
            world: Scene::load_scene_from_uri(&project, &project.manifest().default_scene)?,
            editor_workspace: load_editor_workspace(&root)?,
        })
    }

    pub fn save_to_path(
        path: impl AsRef<Path>,
        world: &Scene,
        editor_workspace: Option<&ProjectEditorWorkspace>,
    ) -> Result<(), SceneProjectError> {
        let root = project_root_path(path)?;
        Self::ensure_runtime_assets(&root)?;

        let mut project = ProjectManager::open(&root)?;
        project.scan_and_import()?;
        let scene = world.to_scene_asset(&project)?;
        let scene_path = project.source_path_for_uri(&project.manifest().default_scene)?;
        if let Some(parent) = scene_path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }
        fs::write(
            scene_path,
            scene
                .to_toml_string()
                .map_err(|error| invalid_data(error.to_string()))?,
        )?;
        save_editor_workspace(&root, editor_workspace)?;
        Ok(())
    }

    pub fn ensure_runtime_assets(root: impl AsRef<Path>) -> Result<(), SceneProjectError> {
        let root = project_root_path(root)?;
        let paths = ProjectPaths::from_root(&root)?;
        paths.ensure_layout()?;

        if !paths.manifest_path().exists() {
            let default_scene = parse_asset_uri(DEFAULT_SCENE_URI)?;
            let project_name = root
                .file_name()
                .and_then(OsStr::to_str)
                .filter(|value| !value.is_empty())
                .unwrap_or("ZirconProject");
            ProjectManifest::new(project_name, default_scene, 1).save(paths.manifest_path())?;
        }

        write_if_missing(
            paths.assets_root().join("shaders").join("pbr.wgsl"),
            DEFAULT_PBR_WGSL,
        )?;
        write_if_missing(
            paths.assets_root()
                .join("materials")
                .join("default.material.toml"),
            default_material_asset()
                .to_toml_string()
                .map_err(|error| invalid_data(error.to_string()))?,
        )?;
        write_if_missing(paths.assets_root().join("models").join("cube.obj"), DEFAULT_CUBE_OBJ)?;

        Ok(())
    }
}

pub fn project_root_path(path: impl AsRef<Path>) -> Result<PathBuf, SceneProjectError> {
    let candidate = path.as_ref();
    let root = if candidate
        .file_name()
        .is_some_and(|name| name == OsStr::new("zircon-project.toml"))
    {
        candidate.parent().unwrap_or(candidate)
    } else {
        candidate
    };
    if root.is_absolute() {
        Ok(root.to_path_buf())
    } else {
        Ok(std::env::current_dir()?.join(root))
    }
}

pub(crate) fn save_layout_preset_asset(
    root: impl AsRef<Path>,
    name: &str,
    layout: &WorkbenchLayout,
) -> Result<PathBuf, SceneProjectError> {
    let root = project_root_path(root)?;
    let path = layout_preset_asset_path(&root, name)?;
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    let document = LayoutPresetAssetDocument {
        format_version: EDITOR_LAYOUT_PRESET_FORMAT_VERSION,
        preset_name: name.to_string(),
        workbench: layout.clone(),
    };
    fs::write(&path, serde_json::to_string_pretty(&document)?)?;
    Ok(path)
}

pub(crate) fn load_layout_preset_asset(
    root: impl AsRef<Path>,
    name: &str,
) -> Result<Option<WorkbenchLayout>, SceneProjectError> {
    let root = project_root_path(root)?;
    let path = layout_preset_asset_path(&root, name)?;
    if !path.exists() {
        return Ok(None);
    }
    let document = serde_json::from_str::<LayoutPresetAssetDocument>(&fs::read_to_string(path)?)?;
    Ok(Some(document.workbench))
}

pub(crate) fn list_layout_preset_assets(
    root: impl AsRef<Path>,
) -> Result<Vec<String>, SceneProjectError> {
    let root = project_root_path(root)?;
    let paths = ProjectPaths::from_root(&root)?;
    let preset_dir = paths.assets_root().join(EDITOR_LAYOUT_PRESET_DIR);
    if !preset_dir.exists() {
        return Ok(Vec::new());
    }

    let mut preset_names = Vec::new();
    for entry in fs::read_dir(preset_dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if !path
            .file_name()
            .and_then(|value| value.to_str())
            .is_some_and(|name| name.ends_with(EDITOR_LAYOUT_PRESET_SUFFIX))
        {
            continue;
        }
        let name = fs::read_to_string(&path)
            .ok()
            .and_then(|contents| serde_json::from_str::<LayoutPresetAssetDocument>(&contents).ok())
            .map(|document| document.preset_name)
            .or_else(|| {
                path.file_name()
                    .and_then(|value| value.to_str())
                    .map(|value| value.trim_end_matches(EDITOR_LAYOUT_PRESET_SUFFIX).to_string())
            });
        if let Some(name) = name {
            preset_names.push(name);
        }
    }
    preset_names.sort();
    preset_names.dedup();
    Ok(preset_names)
}

fn workspace_document_path(root: &Path) -> PathBuf {
    root.join(EDITOR_WORKSPACE_DIR).join(EDITOR_WORKSPACE_FILE)
}

fn layout_preset_asset_path(root: &Path, name: &str) -> Result<PathBuf, SceneProjectError> {
    let paths = ProjectPaths::from_root(root)?;
    Ok(paths
        .assets_root()
        .join(EDITOR_LAYOUT_PRESET_DIR)
        .join(format!("{}{}", sanitize_layout_preset_name(name), EDITOR_LAYOUT_PRESET_SUFFIX)))
}

fn sanitize_layout_preset_name(name: &str) -> String {
    let sanitized = name
        .chars()
        .map(|ch| match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' => ch,
            _ => '-',
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string();
    if sanitized.is_empty() {
        "preset".to_string()
    } else {
        sanitized
    }
}

fn load_editor_workspace(root: &Path) -> Result<Option<ProjectEditorWorkspace>, SceneProjectError> {
    let path = workspace_document_path(root);
    if !path.exists() {
        return Ok(None);
    }
    let document = serde_json::from_str::<EditorWorkspaceDocument>(&fs::read_to_string(path)?)?;
    Ok(Some(document.editor_workspace))
}

fn save_editor_workspace(
    root: &Path,
    editor_workspace: Option<&ProjectEditorWorkspace>,
) -> Result<(), SceneProjectError> {
    let path = workspace_document_path(root);
    if let Some(workspace) = editor_workspace {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }
        let document = EditorWorkspaceDocument {
            format_version: EDITOR_PROJECT_FORMAT_VERSION,
            editor_workspace: workspace.clone(),
        };
        fs::write(path, serde_json::to_string_pretty(&document)?)?;
    } else if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}

fn write_if_missing(path: PathBuf, contents: impl AsRef<[u8]>) -> Result<(), SceneProjectError> {
    if path.exists() {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    fs::write(path, contents)?;
    Ok(())
}

fn default_material_asset() -> MaterialAsset {
    MaterialAsset {
        name: Some("Default".to_string()),
        shader: AssetReference::from_locator(
            ResourceLocator::parse(DEFAULT_SHADER_URI).expect("default shader uri"),
        ),
        base_color: [0.85, 0.85, 0.85, 1.0],
        base_color_texture: None,
        normal_texture: None,
        metallic: 0.0,
        roughness: 1.0,
        metallic_roughness_texture: None,
        occlusion_texture: None,
        emissive: [0.0, 0.0, 0.0],
        emissive_texture: None,
        alpha_mode: AlphaMode::Opaque,
        double_sided: false,
    }
}

fn parse_asset_uri(value: &str) -> Result<ResourceLocator, SceneProjectError> {
    ResourceLocator::parse(value)
        .map_err(|error| SceneProjectError::Asset(AssetImportError::from(error)))
}

fn invalid_data(message: impl Into<String>) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, message.into())
}

const DEFAULT_PBR_WGSL: &str = r#"
struct SceneUniform {
    view_proj: mat4x4<f32>,
    light_dir: vec4<f32>,
    light_color: vec4<f32>,
};

struct ModelUniform {
    model: mat4x4<f32>,
    tint: vec4<f32>,
};

@group(0) @binding(0) var<uniform> scene: SceneUniform;
@group(1) @binding(0) var<uniform> model: ModelUniform;
@group(2) @binding(0) var color_texture: texture_2d<f32>;
@group(2) @binding(1) var color_sampler: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) world_normal: vec3<f32>,
    @location(1) uv: vec2<f32>,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let world_position = model.model * vec4<f32>(input.position, 1.0);
    out.position = scene.view_proj * world_position;
    out.world_normal = normalize((model.model * vec4<f32>(input.normal, 0.0)).xyz);
    out.uv = input.uv;
    return out;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let albedo = textureSample(color_texture, color_sampler, input.uv) * model.tint;
    let ndotl = max(dot(normalize(input.world_normal), normalize(-scene.light_dir.xyz)), 0.0);
    let lighting = 0.15 + ndotl;
    return vec4<f32>(albedo.rgb * scene.light_color.rgb * lighting, albedo.a);
}
"#;

const DEFAULT_CUBE_OBJ: &str = "\
v -0.5 -0.5 0.5
v 0.5 -0.5 0.5
v 0.5 0.5 0.5
v -0.5 0.5 0.5
v -0.5 -0.5 -0.5
v 0.5 -0.5 -0.5
v 0.5 0.5 -0.5
v -0.5 0.5 -0.5
vt 0.0 0.0
vt 1.0 0.0
vt 1.0 1.0
vt 0.0 1.0
vn 0.0 0.0 1.0
vn 0.0 0.0 -1.0
vn 0.0 1.0 0.0
vn 0.0 -1.0 0.0
vn 1.0 0.0 0.0
vn -1.0 0.0 0.0
f 1/1/1 2/2/1 3/3/1
f 1/1/1 3/3/1 4/4/1
f 6/1/2 5/2/2 8/3/2
f 6/1/2 8/3/2 7/4/2
f 4/1/3 3/2/3 7/3/3
f 4/1/3 7/3/3 8/4/3
f 5/1/4 6/2/4 2/3/4
f 5/1/4 2/3/4 1/4/4
f 2/1/5 6/2/5 7/3/5
f 2/1/5 7/3/5 3/4/5
f 5/1/6 1/2/6 4/3/6
f 5/1/6 4/3/6 8/4/6
";
