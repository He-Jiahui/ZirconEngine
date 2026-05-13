use crate::core::framework::render::{
    FallbackSkyboxKind, PreviewEnvironmentExtract, RenderFrameExtract, RenderOverlayExtract,
    RenderSceneGeometryExtract, RenderSceneSnapshot, RenderWorldSnapshotHandle,
    ViewportCameraSnapshot,
};
use crate::core::math::UVec2;
use crate::ui::dispatch::{UiNavigationDispatcher, UiPointerDispatcher};
use crate::ui::surface::UiSurface;
use crate::ui::v2::{UiV2PrototypeStoreFileCache, UiV2SurfaceBuilder};
use zircon_runtime_interface::ui::tree::UiTreeError;
use zircon_runtime_interface::ui::{
    dispatch::{UiNavigationDispatchResult, UiPointerDispatchResult, UiPointerEvent},
    event_ui::UiTreeId,
    layout::UiSize,
    surface::UiNavigationEventKind,
};

use super::public_frame::PublicRuntimeFrame;
use super::runtime_ui_fixture::RuntimeUiFixture;
use super::runtime_ui_manager_error::RuntimeUiManagerError;

pub(crate) struct RuntimeUiManager {
    viewport_size: UVec2,
    fixture_cache: UiV2PrototypeStoreFileCache,
    surface: UiSurface,
    active_fixture: Option<RuntimeUiFixture>,
}

impl RuntimeUiManager {
    pub(crate) fn new(viewport_size: UVec2) -> Self {
        Self {
            viewport_size: UVec2::new(viewport_size.x.max(1), viewport_size.y.max(1)),
            fixture_cache: UiV2PrototypeStoreFileCache::new(),
            surface: UiSurface::new(UiTreeId::new("runtime.ui.empty")),
            active_fixture: None,
        }
    }

    pub(crate) fn load_builtin_fixture(
        &mut self,
        fixture: RuntimeUiFixture,
    ) -> Result<(), RuntimeUiManagerError> {
        let outcome = self
            .fixture_cache
            .load_store(std::iter::once(fixture.asset_path()))?;
        let mut surface = UiV2SurfaceBuilder::build_surface_from_compiled_document(
            UiTreeId::new(fixture.asset_id()),
            outcome.root_document.as_ref(),
            outcome.compiled.as_ref(),
        )?;
        surface.compute_layout(self.root_size())?;

        self.surface = surface;
        self.active_fixture = Some(fixture);
        Ok(())
    }

    pub(crate) fn surface(&self) -> &UiSurface {
        &self.surface
    }

    pub(crate) fn dispatch_pointer_event(
        &mut self,
        dispatcher: &UiPointerDispatcher,
        event: UiPointerEvent,
    ) -> Result<UiPointerDispatchResult, UiTreeError> {
        let result = self.surface.dispatch_pointer_event(dispatcher, event)?;
        self.surface.apply_pointer_dispatch_dirty(&result)?;
        self.rebuild_dirty_surface()?;
        Ok(result)
    }

    pub(crate) fn dispatch_navigation_event(
        &mut self,
        dispatcher: &UiNavigationDispatcher,
        kind: UiNavigationEventKind,
    ) -> Result<UiNavigationDispatchResult, UiTreeError> {
        let result = self.surface.dispatch_navigation_event(dispatcher, kind)?;
        self.rebuild_dirty_surface()?;
        Ok(result)
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

    fn rebuild_dirty_surface(&mut self) -> Result<(), UiTreeError> {
        if self.surface.dirty_flags().any() {
            self.surface.rebuild_dirty(self.root_size())?;
        }
        Ok(())
    }

    fn root_size(&self) -> UiSize {
        UiSize::new(self.viewport_size.x as f32, self.viewport_size.y as f32)
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
