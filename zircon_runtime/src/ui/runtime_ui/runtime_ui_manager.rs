use crate::core::framework::render::{
    FallbackSkyboxKind, PreviewEnvironmentExtract, RenderFrameExtract, RenderOverlayExtract,
    RenderSceneGeometryExtract, RenderSceneSnapshot, RenderWorldSnapshotHandle,
    ViewportCameraSnapshot,
};
use crate::core::math::UVec2;
use crate::ui::template::{UiAssetLoader, UiDocumentCompiler, UiTemplateSurfaceBuilder};
use crate::ui::{event_ui::UiTreeId, layout::UiSize, surface::UiSurface};

use super::public_frame::PublicRuntimeFrame;
use super::runtime_ui_fixture::RuntimeUiFixture;
use super::runtime_ui_manager_error::RuntimeUiManagerError;

pub(crate) struct RuntimeUiManager {
    viewport_size: UVec2,
    compiler: UiDocumentCompiler,
    surface: UiSurface,
    active_fixture: Option<RuntimeUiFixture>,
}

impl RuntimeUiManager {
    pub(crate) fn new(viewport_size: UVec2) -> Self {
        Self {
            viewport_size: UVec2::new(viewport_size.x.max(1), viewport_size.y.max(1)),
            compiler: UiDocumentCompiler::default(),
            surface: UiSurface::new(UiTreeId::new("runtime.ui.empty")),
            active_fixture: None,
        }
    }

    pub(crate) fn load_builtin_fixture(
        &mut self,
        fixture: RuntimeUiFixture,
    ) -> Result<(), RuntimeUiManagerError> {
        let document = UiAssetLoader::load_toml_file(fixture.asset_path())?;
        let compiled = self.compiler.compile(&document)?;
        let mut surface = UiTemplateSurfaceBuilder::build_surface_from_compiled_document(
            UiTreeId::new(fixture.asset_id()),
            &compiled,
        )?;
        surface.compute_layout(UiSize::new(
            self.viewport_size.x as f32,
            self.viewport_size.y as f32,
        ))?;

        self.surface = surface;
        self.active_fixture = Some(fixture);
        Ok(())
    }

    pub(crate) fn surface(&self) -> &UiSurface {
        &self.surface
    }

    pub(crate) fn build_frame(&self) -> PublicRuntimeFrame {
        let extract = RenderFrameExtract::from_snapshot(
            RenderWorldSnapshotHandle::new(0),
            empty_scene_snapshot(self.viewport_size),
        );
        PublicRuntimeFrame {
            extract,
            viewport_size: self.viewport_size,
            ui: Some(self.surface.render_extract.clone()),
        }
    }

    pub(crate) fn active_fixture(&self) -> Option<RuntimeUiFixture> {
        self.active_fixture
    }
}

fn empty_scene_snapshot(viewport_size: UVec2) -> RenderSceneSnapshot {
    let mut camera = ViewportCameraSnapshot::default();
    camera.apply_viewport_size(viewport_size);

    RenderSceneSnapshot {
        scene: RenderSceneGeometryExtract {
            camera,
            meshes: Vec::new(),
            directional_lights: Vec::new(),
            point_lights: Vec::new(),
            spot_lights: Vec::new(),
        },
        overlays: RenderOverlayExtract::default(),
        preview: PreviewEnvironmentExtract {
            lighting_enabled: false,
            skybox_enabled: false,
            fallback_skybox: FallbackSkyboxKind::None,
            clear_color: crate::core::math::Vec4::new(0.02, 0.02, 0.03, 1.0),
        },
        virtual_geometry_debug: None,
    }
}
