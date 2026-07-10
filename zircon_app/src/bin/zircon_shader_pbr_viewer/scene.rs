use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use zircon_runtime::asset::pipeline::manager::{AssetManager, ProjectAssetManager};
use zircon_runtime::asset::project::{ProjectManager, ProjectManifest, ProjectPaths};
use zircon_runtime::asset::AssetUri;
use zircon_runtime::core::framework::render::{
    EnvironmentExtract, PreviewEnvironmentExtract, RenderOverlayExtract,
    SceneViewportExtractRequest, ViewportRenderSettings,
};
use zircon_runtime::core::math::{UVec2, Vec4};
use zircon_runtime::graphics::{SceneRenderer, ViewportFrame};

use crate::camera::{camera_render_descriptor, OrbitCamera};
use crate::hdri::source_cubemap_environment;
use crate::project_assets::write_viewer_project_assets;

pub(crate) struct PbrMirrorScene {
    project_root: PathBuf,
    world: zircon_runtime::scene::world::World,
    renderer: SceneRenderer,
    environment: EnvironmentExtract,
}

impl PbrMirrorScene {
    pub(crate) fn new(hdri_path: &Path, face_size: u32) -> Result<Self, Box<dyn Error>> {
        let project_root = unique_temp_project_root("shader_pbr_viewer");
        let paths = ProjectPaths::from_root(&project_root)?;
        paths.ensure_layout()?;
        let scene_uri = AssetUri::parse("res://scenes/single_pbr_sphere.scene.toml")?;
        ProjectManifest::new("ShaderPbrMirrorViewer", scene_uri.clone(), 1)
            .save(paths.manifest_path())?;
        write_viewer_project_assets(&paths)?;

        let asset_manager = Arc::new(ProjectAssetManager::default());
        asset_manager.open_project(project_root.to_string_lossy().as_ref())?;
        let mut project = ProjectManager::open(&project_root)?;
        project.scan_and_import()?;
        let world = zircon_runtime::scene::world::World::load_scene_from_uri(&project, &scene_uri)?;
        let renderer = SceneRenderer::new(asset_manager)?;
        let environment = EnvironmentExtract::source_cubemap(source_cubemap_environment(
            hdri_path,
            face_size,
            paths.library_root(),
        )?);

        Ok(Self {
            project_root,
            world,
            renderer,
            environment,
        })
    }

    pub(crate) fn render(
        &mut self,
        camera: &OrbitCamera,
        viewport_size: UVec2,
    ) -> Result<ViewportFrame, Box<dyn Error>> {
        let camera_descriptor = camera_render_descriptor(camera, viewport_size);
        let mut snapshot = self
            .world
            .build_viewport_render_packet(&SceneViewportExtractRequest {
                settings: ViewportRenderSettings::default(),
                active_camera_override: None,
                camera: Some(camera_descriptor),
                viewport_size: Some(viewport_size),
                virtual_geometry_debug: None,
            });
        snapshot.environment = self.environment.clone();
        snapshot.preview =
            PreviewEnvironmentExtract::from_environment(&snapshot.environment, true, Vec4::ZERO);
        snapshot.overlays = RenderOverlayExtract::default();

        Ok(self.renderer.render(snapshot, viewport_size)?)
    }

    pub(crate) fn renderer_backend_name(&self) -> &str {
        self.renderer.backend_name()
    }

    pub(crate) fn start_graphics_debugger_capture(&self) {
        self.renderer.start_graphics_debugger_capture();
    }

    pub(crate) fn stop_graphics_debugger_capture(&self) -> Result<(), Box<dyn Error>> {
        Ok(self.renderer.stop_graphics_debugger_capture()?)
    }
}

impl Drop for PbrMirrorScene {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.project_root);
    }
}

fn unique_temp_project_root(label: &str) -> PathBuf {
    static NEXT_TEMP_PROJECT_ID: AtomicU64 = AtomicU64::new(1);

    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let process_id = std::process::id();
    let sequence = NEXT_TEMP_PROJECT_ID.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("zircon_{label}_{process_id}_{sequence}_{unique}"))
}
