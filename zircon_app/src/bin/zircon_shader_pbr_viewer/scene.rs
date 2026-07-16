use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use zircon_runtime::asset::pipeline::manager::{
    project_asset_manager_handle, AssetManager, ProjectAssetManagerAccess,
};
use zircon_runtime::asset::project::{ProjectManager, ProjectManifest, ProjectPaths};
use zircon_runtime::asset::AssetUri;
use zircon_runtime::core::framework::render::{
    EnvironmentExtract, PreviewEnvironmentExtract, RenderOverlayExtract,
    SceneViewportExtractRequest, ViewportRenderSettings,
};
use zircon_runtime::core::math::{UVec2, Vec4};
use zircon_runtime::core::runtime::modules::{TasksModule, TASKS_MODULE_NAME};
use zircon_runtime::core::CoreRuntime;
use zircon_runtime::engine_module::EngineModule;
use zircon_runtime::graphics::{SceneRenderer, ViewportFrame};

use crate::camera::{camera_render_descriptor, OrbitCamera};
use crate::hdri::source_cubemap_environment;
use crate::project_assets::write_viewer_project_assets;

pub(crate) struct PbrMirrorScene {
    project_root: PathBuf,
    world: zircon_runtime::scene::world::World,
    renderer: SceneRenderer,
    environment: EnvironmentExtract,
    // Rust drops fields in declaration order; services must outlive renderer teardown.
    _asset_runtime: CoreRuntime,
}

impl PbrMirrorScene {
    // The HDRI loader resolves an omitted face size after decoding the source dimensions.
    pub(crate) fn new(
        hdri_path: &Path,
        face_size: Option<u32>,
        pmrem_face_size: Option<u32>,
    ) -> Result<Self, Box<dyn Error>> {
        let project_root = unique_temp_project_root("shader_pbr_viewer");
        let paths = ProjectPaths::from_root(&project_root)?;
        let scene_uri = AssetUri::parse("res://scenes/single_pbr_sphere.scene.toml")?;
        let manifest = ProjectManifest::new("ShaderPbrMirrorViewer", scene_uri.clone(), 1);
        paths.ensure_layout(&manifest.asset_roots)?;
        manifest.save(paths.manifest_path())?;
        let asset_root = manifest.primary_asset_root_path(&paths)?;
        write_viewer_project_assets(&asset_root)?;

        let asset_runtime = CoreRuntime::new();
        asset_runtime.register_module(zircon_runtime::foundation::module_descriptor())?;
        asset_runtime.register_module(TasksModule.descriptor())?;
        asset_runtime.register_module(zircon_runtime::asset::module_descriptor())?;
        asset_runtime.activate_module(zircon_runtime::foundation::FOUNDATION_MODULE_NAME)?;
        asset_runtime.activate_module(TASKS_MODULE_NAME)?;
        asset_runtime.activate_module(zircon_runtime::asset::ASSET_MODULE_NAME)?;
        let core = asset_runtime.handle();
        let asset_access =
            ProjectAssetManagerAccess::new(core.clone(), project_asset_manager_handle(&core)?);
        let asset_manager = asset_access.resolve()?;
        asset_manager.open_project(project_root.to_string_lossy().as_ref())?;
        let mut project = ProjectManager::open(&project_root)?;
        project.scan_and_import()?;
        let world = zircon_runtime::scene::world::World::load_scene_from_uri(&project, &scene_uri)?;
        let renderer = SceneRenderer::new(asset_access)?;
        let environment = EnvironmentExtract::source_cubemap(source_cubemap_environment(
            hdri_path,
            face_size,
            pmrem_face_size,
            paths.cache_root(),
        )?);

        Ok(Self {
            project_root,
            world,
            renderer,
            environment,
            _asset_runtime: asset_runtime,
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

#[cfg(test)]
mod tests {
    const SOURCE: &str = include_str!("scene.rs");

    fn production_source() -> &'static str {
        SOURCE
            .split_once("#[cfg(test)]")
            .map(|(production, _)| production)
            .expect("scene source should retain a test-module boundary")
    }

    fn assert_source_order(anchors: &[&str]) {
        let source = production_source();
        let mut offset = 0;
        for anchor in anchors {
            let relative = source[offset..]
                .find(anchor)
                .unwrap_or_else(|| panic!("missing viewer architecture anchor: {anchor}"));
            offset += relative + anchor.len();
        }
    }

    #[test]
    fn scene_renderer_drops_before_its_runtime_services() {
        let fields = production_source()
            .split_once("pub(crate) struct PbrMirrorScene {")
            .and_then(|(_, rest)| rest.split_once('}'))
            .map(|(fields, _)| fields)
            .expect("PbrMirrorScene fields should remain visible to the lifecycle guard");

        assert!(
            fields.find("renderer:").expect("renderer field")
                < fields
                    .find("_asset_runtime:")
                    .expect("runtime owner field"),
            "Rust drops struct fields in declaration order, so the runtime owner must follow the renderer"
        );
    }

    #[test]
    fn viewer_uses_real_runtime_module_and_asset_manager_lifecycle() {
        let source = production_source();
        assert!(
            !source.contains("fn viewer_uses_real_runtime_module_and_asset_manager_lifecycle"),
            "architecture guards must never search their own anchor strings"
        );
        assert_source_order(&[
            "register_module(zircon_runtime::foundation::module_descriptor())",
            "register_module(TasksModule.descriptor())",
            "register_module(zircon_runtime::asset::module_descriptor())",
            "activate_module(zircon_runtime::foundation::FOUNDATION_MODULE_NAME)",
            "activate_module(TASKS_MODULE_NAME)",
            "activate_module(zircon_runtime::asset::ASSET_MODULE_NAME)",
            "ProjectAssetManagerAccess::new",
            "SceneRenderer::new(asset_access)",
        ]);
        assert!(
            !source.contains(concat!("ProjectAssetManager::", "default")),
            "viewer must not construct a direct default asset manager"
        );
        assert!(
            !source.contains(concat!("new_", "for_test")),
            "viewer must not use a test-only renderer path"
        );
    }
}
