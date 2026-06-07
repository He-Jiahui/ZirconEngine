use std::fs;
use std::path::{Path, PathBuf};

use crate::error::HubError;

use super::{CreateProjectRequest, ProjectTemplate};

const PROJECT_MANIFEST_FILE: &str = "zircon-project.toml";
const DEFAULT_SCENE_PATH: &str = "scenes/main.scene.toml";
const DEFAULT_SCENE_URI: &str = "res://scenes/main.scene.toml";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CreateProjectReport {
    pub project_root: PathBuf,
    pub manifest_path: PathBuf,
    pub template: ProjectTemplate,
}

pub fn create_project(request: &CreateProjectRequest) -> Result<CreateProjectReport, HubError> {
    request
        .validate_launch_fields()
        .map_err(HubError::message)?;
    match request.template {
        ProjectTemplate::RenderableEmpty => create_renderable_empty_project(request),
    }
}

fn create_renderable_empty_project(
    request: &CreateProjectRequest,
) -> Result<CreateProjectReport, HubError> {
    let project_root = request.target_root();
    validate_target_root(&project_root)?;
    fs::create_dir_all(&project_root)?;
    create_standard_project_layout(&project_root)?;
    write_manifest(&project_root, &request.project_name)?;
    write_default_scene(&project_root)?;
    Ok(CreateProjectReport {
        manifest_path: project_root.join(PROJECT_MANIFEST_FILE),
        project_root,
        template: request.template,
    })
}

fn validate_target_root(project_root: &Path) -> Result<(), HubError> {
    if project_root.as_os_str().is_empty() {
        return Err(HubError::message("Project root is required"));
    }
    if project_root.is_file() {
        return Err(HubError::message("Target path already exists as a file"));
    }
    if project_root.is_dir() && project_root.read_dir()?.next().transpose()?.is_some() {
        return Err(HubError::message("Target directory must be empty"));
    }
    Ok(())
}

fn create_standard_project_layout(project_root: &Path) -> Result<(), HubError> {
    for relative in [
        "assets/scenes",
        "assets/materials",
        "assets/models",
        "assets/shaders/pbr_shader",
        "library",
    ] {
        fs::create_dir_all(project_root.join(relative))?;
    }
    write_if_missing(
        project_root
            .join("assets")
            .join("materials")
            .join("default.zmaterial"),
        DEFAULT_MATERIAL,
    )?;
    write_if_missing(
        project_root.join("assets").join("models").join("cube.obj"),
        DEFAULT_CUBE_OBJ,
    )?;
    write_if_missing(
        project_root
            .join("assets")
            .join("shaders")
            .join("pbr_shader.zmeta"),
        DEFAULT_SHADER_META,
    )?;
    write_if_missing(
        project_root
            .join("assets")
            .join("shaders")
            .join("pbr_shader")
            .join("pbr.zshader"),
        DEFAULT_PBR_ZSHADER,
    )?;
    write_if_missing(
        project_root
            .join("assets")
            .join("shaders")
            .join("pbr_shader")
            .join("pbr.wgsl"),
        DEFAULT_PBR_WGSL,
    )?;
    Ok(())
}

fn write_manifest(project_root: &Path, project_name: &str) -> Result<(), HubError> {
    write_if_missing(
        project_root.join(PROJECT_MANIFEST_FILE),
        format!(
            "name = \"{}\"\nformat_version = 1\ndefault_scene = \"{}\"\nlibrary_version = 1\n",
            escape_toml_string(project_name),
            DEFAULT_SCENE_URI
        ),
    )
}

fn write_default_scene(project_root: &Path) -> Result<(), HubError> {
    write_if_missing(
        project_root.join("assets").join(DEFAULT_SCENE_PATH),
        DEFAULT_SCENE,
    )
}

fn write_if_missing(path: PathBuf, contents: impl AsRef<[u8]>) -> Result<(), HubError> {
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

fn escape_toml_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

const DEFAULT_SCENE: &str = r#"[[entities]]
entity = 1
name = "Camera"
parent = 0
transform = { translation = [0.0, 2.0, 5.0], rotation = [0.0, 0.0, 0.0, 1.0], scale = [1.0, 1.0, 1.0] }
camera = { fov_y_radians = 1.0471976, z_near = 0.1, z_far = 200.0 }

[[entities]]
entity = 2
name = "Sun"
parent = 0
transform = { translation = [0.0, 4.0, 0.0], rotation = [0.0, 0.0, 0.0, 1.0], scale = [1.0, 1.0, 1.0] }
directional_light = { direction = [-0.4, -1.0, -0.25], color = [1.0, 1.0, 1.0], intensity = 3.0 }
"#;

const DEFAULT_MATERIAL: &str = r#"name = "Default"
shader = { uuid = "00000000-0000-0000-0000-000000000000", locator = "res://shaders/pbr_shader" }
base_color = [0.85, 0.85, 0.85, 1.0]
metallic = 0.0
roughness = 1.0
emissive = [0.0, 0.0, 0.0]
alpha_mode = "opaque"
double_sided = false
"#;

const DEFAULT_SHADER_META: &str = r#"format_version = 6
uuid = "00000000-0000-0000-0000-000000000001"
url = "res://shaders/pbr_shader"
asset_kind = "shader"
unit = "compound"
included_files = []
importer_id = ""
import_settings = {}
config_hash = ""
source_mtime_unix_ms = 0
source_hash = ""
preview_state = "dirty"
importer_version = 0
migration_summary = ""
dependencies = []
entries = []
"#;

const DEFAULT_PBR_ZSHADER: &str = r#"version = 1
name = "Default PBR"
import_path = "zircon::pbr"
wgsl_files = ["pbr.wgsl"]

[[entry_points]]
name = "vs_main"
stage = "vertex"
file = "pbr.wgsl"

[[entry_points]]
name = "fs_main"
stage = "fragment"
file = "pbr.wgsl"

[[properties]]
name = "base_color"
kind = "vec4"
default = [1.0, 1.0, 1.0, 1.0]
editor = { label = "Base Color", group = "Surface" }
"#;

const DEFAULT_PBR_WGSL: &str = r#"struct SceneUniform {
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
    let ndotl = max(dot(normalize(input.world_normal), normalize(-scene.light_dir.xyz)), 0.0);
    let lighting = 0.15 + ndotl;
    return vec4<f32>(vec3<f32>(lighting), 1.0);
}
"#;

const DEFAULT_CUBE_OBJ: &str = r#"v -0.5 -0.5 0.5
v 0.5 -0.5 0.5
v 0.5 0.5 0.5
v -0.5 0.5 0.5
v -0.5 -0.5 -0.5
v 0.5 -0.5 -0.5
v 0.5 0.5 -0.5
v -0.5 0.5 -0.5
f 1 2 3
f 1 3 4
f 6 5 8
f 6 8 7
"#;

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    #[test]
    fn create_project_writes_manifest_and_standard_local_layout() {
        let location = temp_dir("zircon-hub-create-project");
        let request =
            CreateProjectRequest::new("My Game", &location, ProjectTemplate::RenderableEmpty);

        let report = create_project(&request).expect("project should be scaffolded");

        assert_eq!(report.project_root, location.join("My Game"));
        assert!(report.manifest_path.is_file());
        assert!(report
            .project_root
            .join("assets")
            .join(DEFAULT_SCENE_PATH)
            .is_file());
        assert!(report.project_root.join("library").is_dir());
        let manifest = fs::read_to_string(report.manifest_path).unwrap();
        assert!(manifest.contains("name = \"My Game\""));
        assert!(manifest.contains("default_scene = \"res://scenes/main.scene.toml\""));

        fs::remove_dir_all(location).unwrap();
    }

    #[test]
    fn create_project_rejects_non_empty_target_directory() {
        let location = temp_dir("zircon-hub-create-project-non-empty");
        let target = location.join("My Game");
        fs::create_dir_all(&target).unwrap();
        fs::write(target.join("existing.txt"), "keep").unwrap();
        let request =
            CreateProjectRequest::new("My Game", &location, ProjectTemplate::RenderableEmpty);

        let error = create_project(&request).unwrap_err();

        assert!(error.to_string().contains("Target directory must be empty"));
        fs::remove_dir_all(location).unwrap();
    }

    fn temp_dir(prefix: &str) -> PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("{prefix}-{suffix}"));
        fs::create_dir_all(&path).unwrap();
        path
    }
}
