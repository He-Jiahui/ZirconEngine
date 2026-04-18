use zircon_math::UVec2;
use zircon_scene::{
    FallbackSkyboxKind, PreviewEnvironmentExtract, RenderFrameExtract, RenderOverlayExtract,
    RenderSceneGeometryExtract, RenderSceneSnapshot, RenderWorldSnapshotHandle,
    ViewportCameraSnapshot,
};
use zircon_ui::{
    UiAssetLoader, UiDocumentCompiler, UiSize, UiSurface, UiTemplateSurfaceBuilder, UiTreeId,
};

use crate::EditorOrRuntimeFrame;

use super::runtime_ui_fixture::RuntimeUiFixture;
use super::runtime_ui_manager_error::RuntimeUiManagerError;

pub struct RuntimeUiManager {
    viewport_size: UVec2,
    compiler: UiDocumentCompiler,
    surface: UiSurface,
    active_fixture: Option<RuntimeUiFixture>,
}

impl RuntimeUiManager {
    pub fn new(viewport_size: UVec2) -> Self {
        Self {
            viewport_size: UVec2::new(viewport_size.x.max(1), viewport_size.y.max(1)),
            compiler: UiDocumentCompiler::default(),
            surface: UiSurface::new(UiTreeId::new("runtime.ui.empty")),
            active_fixture: None,
        }
    }

    pub fn load_builtin_fixture(
        &mut self,
        fixture: RuntimeUiFixture,
    ) -> Result<(), RuntimeUiManagerError> {
        let document = UiAssetLoader::load_toml_str(fixture.source())?;
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

    pub fn surface(&self) -> &UiSurface {
        &self.surface
    }

    pub fn build_frame(&self) -> EditorOrRuntimeFrame {
        let extract = RenderFrameExtract::from_snapshot(
            RenderWorldSnapshotHandle::new(0),
            empty_scene_snapshot(self.viewport_size),
        );
        EditorOrRuntimeFrame::from_extract(extract, self.viewport_size)
            .with_ui(Some(self.surface.render_extract.clone()))
    }

    pub fn active_fixture(&self) -> Option<RuntimeUiFixture> {
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
            lights: Vec::new(),
        },
        overlays: RenderOverlayExtract::default(),
        preview: PreviewEnvironmentExtract {
            lighting_enabled: false,
            skybox_enabled: false,
            fallback_skybox: FallbackSkyboxKind::None,
            clear_color: zircon_math::Vec4::new(0.02, 0.02, 0.03, 1.0),
        },
    }
}
