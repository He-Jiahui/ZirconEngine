//! 测试管理器贯通 V2 资产、UiInputManager、窗口适配和公开帧；每次输入后重建脏表面，批次保留前序成功结果。

use crate::asset::assets::ui_v2_asset_references;
use crate::core::framework::render::{
    FallbackSkyboxKind, PreviewEnvironmentExtract, RenderFrameExtract, RenderOverlayExtract,
    RenderSceneGeometryExtract, RenderSceneSnapshot, RenderWorldSnapshotHandle, UiRenderSubmission,
    ViewportCameraSnapshot,
};
use crate::core::math::UVec2;
use crate::ui::surface::UiSurface;
use crate::ui::template::UiAssetSurfaceIndex;
use crate::ui::theme::UiThemeRegistry;
use crate::ui::v2::{UiV2PrototypeStoreFileCache, UiV2SurfaceBuilder};
use crate::ui::{dispatch::UiInputManager, PublicRuntimeFrame};
use std::sync::Arc;
use zircon_runtime_interface::ui::tree::UiTreeError;
use zircon_runtime_interface::ui::v2::UiV2AssetDocument;
use zircon_runtime_interface::ui::{
    dispatch::{
        UiDispatchPhase, UiInputDispatchResult, UiInputEvent, UiNavigationDispatchContext,
        UiNavigationDispatchEffect, UiNavigationDispatchResult, UiPointerDispatchContext,
        UiPointerDispatchEffect, UiPointerDispatchResult, UiPointerEvent,
    },
    event_ui::{UiNodeId, UiTreeId},
    layout::UiSize,
    surface::{UiNavigationEventKind, UiPointerEventKind},
    window::{
        UiRuntimeEventAdapterContext, UiWindowEventKind, UiWindowInputPumpBatch,
        UiWindowInputPumpEvent, UiWindowPlatformInputEvent,
    },
};
use zircon_runtime_interface::ZrRuntimeEventV1;

use super::runtime_ui_fixture::RuntimeUiFixture;
use super::runtime_ui_manager_error::RuntimeUiManagerError;

/// 只在 cfg(test) 的 UI 夹具中持有输入管理器和表面，复现资产到公开帧的集成顺序；不参与产品启动。
pub(crate) struct RuntimeUiManager {
    viewport_size: UVec2,
    fixture_cache: UiV2PrototypeStoreFileCache,
    theme_registry: UiThemeRegistry,
    surface: UiSurface,
    asset_surface_index: UiAssetSurfaceIndex,
    input_manager: UiInputManager,
    active_fixture: Option<RuntimeUiFixture>,
}

impl RuntimeUiManager {
    pub(crate) fn new(viewport_size: UVec2) -> Self {
        Self {
            viewport_size: UVec2::new(viewport_size.x.max(1), viewport_size.y.max(1)),
            fixture_cache: UiV2PrototypeStoreFileCache::new(),
            theme_registry: UiThemeRegistry::default(),
            surface: UiSurface::new(UiTreeId::new("runtime.ui.empty")),
            asset_surface_index: UiAssetSurfaceIndex::new(),
            input_manager: UiInputManager::default(),
            active_fixture: None,
        }
    }

    /// 成功加载才替换当前表面；新的节点集合同时重置输入管理器，防止沿用旧处理器或计时器。
    pub(crate) fn load_builtin_fixture(
        &mut self,
        fixture: RuntimeUiFixture,
    ) -> Result<(), RuntimeUiManagerError> {
        let outcome = self
            .fixture_cache
            .load_store(std::iter::once(fixture.asset_path()))?;
        let mut surface = UiV2SurfaceBuilder::build_surface_from_compiled_document_with_theme(
            UiTreeId::new(fixture.asset_id()),
            outcome.root_document.as_ref(),
            outcome.compiled.as_ref(),
            &self.theme_registry,
        )?;
        surface.compute_layout(self.root_size())?;

        self.record_loaded_fixture_surface(fixture, outcome.root_document.as_ref(), &surface);
        self.surface = surface;
        self.input_manager = UiInputManager::default();
        self.active_fixture = Some(fixture);
        Ok(())
    }

    pub(crate) fn surface(&self) -> &UiSurface {
        &self.surface
    }

    pub(crate) fn asset_surface_index(&self) -> &UiAssetSurfaceIndex {
        &self.asset_surface_index
    }

    pub(crate) fn register_pointer_handler<F>(
        &mut self,
        node_id: UiNodeId,
        kind: UiPointerEventKind,
        handler: F,
    ) where
        F: for<'route> Fn(&UiPointerDispatchContext<'route>) -> UiPointerDispatchEffect
            + Send
            + Sync
            + 'static,
    {
        self.input_manager
            .pointer_dispatcher_mut()
            .register(node_id, kind, handler);
    }

    pub(crate) fn register_pointer_phase_handler<F>(
        &mut self,
        node_id: UiNodeId,
        kind: UiPointerEventKind,
        phase: UiDispatchPhase,
        handler: F,
    ) where
        F: for<'route> Fn(&UiPointerDispatchContext<'route>) -> UiPointerDispatchEffect
            + Send
            + Sync
            + 'static,
    {
        self.input_manager
            .pointer_dispatcher_mut()
            .register_phase(node_id, kind, phase, handler);
    }

    pub(crate) fn register_navigation_handler<F>(
        &mut self,
        node_id: UiNodeId,
        kind: UiNavigationEventKind,
        handler: F,
    ) where
        F: for<'route> Fn(&UiNavigationDispatchContext<'route>) -> UiNavigationDispatchEffect
            + Send
            + Sync
            + 'static,
    {
        self.input_manager
            .navigation_dispatcher_mut()
            .register(node_id, kind, handler);
    }

    /// 供旧指针路由接口的夹具使用；需要活动指针表或计时器语义时应调用统一 dispatch_input_event。
    pub(crate) fn dispatch_pointer_event(
        &mut self,
        event: UiPointerEvent,
    ) -> Result<UiPointerDispatchResult, UiTreeError> {
        let result = self
            .surface
            .dispatch_pointer_event(self.input_manager.pointer_dispatcher(), event)?;
        self.surface.apply_pointer_dispatch_dirty(&result)?;
        self.rebuild_dirty_surface()?;
        Ok(result)
    }

    pub(crate) fn dispatch_navigation_event(
        &mut self,
        kind: UiNavigationEventKind,
    ) -> Result<UiNavigationDispatchResult, UiTreeError> {
        let result = self
            .surface
            .dispatch_navigation_event(self.input_manager.navigation_dispatcher(), kind)?;
        self.rebuild_dirty_surface()?;
        Ok(result)
    }

    pub(crate) fn dispatch_input_event(
        &mut self,
        event: UiInputEvent,
    ) -> Result<UiInputDispatchResult, UiTreeError> {
        let result = self
            .surface
            .dispatch_input_event_with_manager(&mut self.input_manager, event)?;
        self.rebuild_dirty_surface()?;
        Ok(result)
    }

    pub(crate) fn dispatch_input_batch(
        &mut self,
        events: impl IntoIterator<Item = UiInputEvent>,
    ) -> Result<Vec<UiInputDispatchResult>, RuntimeUiManagerError> {
        dispatch_manager_batch(
            self,
            events,
            |manager, event| manager.dispatch_input_event(event),
            |index, source| RuntimeUiManagerError::InputBatch { index, source },
        )
    }

    pub(crate) fn dispatch_platform_input_event(
        &mut self,
        event: UiWindowPlatformInputEvent,
    ) -> Result<UiInputDispatchResult, UiTreeError> {
        self.dispatch_input_event(event.normalize())
    }

    pub(crate) fn dispatch_platform_input_batch(
        &mut self,
        events: impl IntoIterator<Item = UiWindowPlatformInputEvent>,
    ) -> Result<Vec<UiInputDispatchResult>, RuntimeUiManagerError> {
        dispatch_manager_batch(
            self,
            events,
            |manager, event| manager.dispatch_platform_input_event(event),
            |index, source| RuntimeUiManagerError::PlatformInputBatch { index, source },
        )
    }

    pub(crate) fn dispatch_window_input_pump_event(
        &mut self,
        event: UiWindowInputPumpEvent,
    ) -> Result<UiInputDispatchResult, UiTreeError> {
        let viewport_size = viewport_size_from_window_event(&event);
        let result = self
            .surface
            .dispatch_window_input_pump_event(&mut self.input_manager, event)?;
        if let Some(viewport_size) = viewport_size {
            self.viewport_size = viewport_size;
        }
        self.rebuild_dirty_surface()?;
        Ok(result)
    }

    /// 逐项重建使同一批次的后续指针命中能看到前项窗口尺寸；不是整批回滚事务。
    pub(crate) fn dispatch_window_input_pump_batch(
        &mut self,
        batch: UiWindowInputPumpBatch,
    ) -> Result<Vec<UiInputDispatchResult>, RuntimeUiManagerError> {
        dispatch_manager_batch(
            self,
            batch.events,
            |manager, event| manager.dispatch_window_input_pump_event(event),
            |index, source| RuntimeUiManagerError::WindowInputPumpBatch { index, source },
        )
    }

    pub(crate) fn dispatch_runtime_event(
        &mut self,
        context: &UiRuntimeEventAdapterContext,
        event: ZrRuntimeEventV1,
    ) -> Result<UiInputDispatchResult, RuntimeUiManagerError> {
        super::window_event::dispatch_runtime_event(self, context, event)
    }

    pub(crate) fn dispatch_runtime_event_batch(
        &mut self,
        context: &UiRuntimeEventAdapterContext,
        events: impl IntoIterator<Item = ZrRuntimeEventV1>,
    ) -> Result<Vec<UiInputDispatchResult>, RuntimeUiManagerError> {
        super::window_event::dispatch_runtime_event_batch(self, context, events)
    }

    /// 为断言生成包含当前 UI 提交的公开帧；空场景只是让夹具无需启动完整渲染宿主。
    pub(crate) fn build_frame(&self) -> PublicRuntimeFrame {
        let extract = RenderFrameExtract::from_snapshot(
            RenderWorldSnapshotHandle::new(0),
            empty_scene_snapshot(self.viewport_size),
        );
        PublicRuntimeFrame {
            extract,
            viewport_size: self.viewport_size,
            ui: Some(UiRenderSubmission::single(Arc::new(
                self.surface.render_extract.clone(),
            ))),
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

    fn record_loaded_fixture_surface(
        &mut self,
        fixture: RuntimeUiFixture,
        root_document: &UiV2AssetDocument,
        surface: &UiSurface,
    ) {
        let mut assets = Vec::new();
        assets.push(fixture.asset_id().to_string());
        assets.push(fixture.asset_uri().to_string());
        assets.extend(
            ui_v2_asset_references(root_document)
                .into_iter()
                .map(|reference| reference.locator.to_string()),
        );
        self.asset_surface_index
            .record_surface_assets(UiTreeId::new(fixture.asset_id()), assets);
        self.asset_surface_index
            .record_tree_node_resources(&surface.tree);
    }
}

fn viewport_size_from_window_event(event: &UiWindowInputPumpEvent) -> Option<UVec2> {
    let UiWindowInputPumpEvent::Window(event) = event else {
        return None;
    };
    match &event.kind {
        UiWindowEventKind::Created { metrics } | UiWindowEventKind::Resized { metrics } => {
            Some(viewport_size_from_metrics(metrics.logical_size))
        }
        _ => None,
    }
}

fn viewport_size_from_metrics(size: UiSize) -> UVec2 {
    UVec2::new(
        sanitized_viewport_axis(size.width),
        sanitized_viewport_axis(size.height),
    )
}

fn sanitized_viewport_axis(value: f32) -> u32 {
    if value.is_finite() && value > 0.0 {
        value.round().max(1.0) as u32
    } else {
        1
    }
}

// 首个失败立即上报原索引，此前成功事件的状态和重建已生效；调用方不可把它理解为原子批次。
fn dispatch_manager_batch<T>(
    manager: &mut RuntimeUiManager,
    events: impl IntoIterator<Item = T>,
    mut dispatch: impl FnMut(&mut RuntimeUiManager, T) -> Result<UiInputDispatchResult, UiTreeError>,
    batch_error: impl Fn(usize, UiTreeError) -> RuntimeUiManagerError,
) -> Result<Vec<UiInputDispatchResult>, RuntimeUiManagerError> {
    let mut results = Vec::new();
    for (index, event) in events.into_iter().enumerate() {
        let result = dispatch(manager, event).map_err(|source| batch_error(index, source))?;
        results.push(result);
    }
    Ok(results)
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
            ambient_lights: Vec::new(),
            rect_lights: Vec::new(),
        },
        overlays: RenderOverlayExtract::default(),
        environment: crate::core::framework::render::EnvironmentExtract::default(),
        preview: PreviewEnvironmentExtract {
            lighting_enabled: false,
            skybox_enabled: false,
            fallback_skybox: FallbackSkyboxKind::None,
            clear_color: crate::core::math::Vec4::new(0.02, 0.02, 0.03, 1.0),
        },
        virtual_geometry_debug: None,
    }
}
