//! 测试管理器贯通 V2 资产、UiInputManager、窗口适配和公开帧；每次输入后重建脏表面，批次保留前序成功结果。
//! 批次首错不交付成功前缀的结果向量；已处理事件的管理器状态仍然保留，不回滚。

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
    /// 创建尚未选择夹具的测试宿主。
    ///
    /// # 参数与返回
    /// `viewport_size` 同时用作布局根尺寸与帧尺寸，各轴下限为 1。
    /// 返回值持有空表面和独立输入状态；加载夹具后，再注册对应节点的回调。
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

    /// 借用当前持久表面供测试断言；返回引用的有效期受 `self` 借用约束。
    /// 持有该引用期间，不能经同一管理器执行需要可变借用的加载或输入处理。
    pub(crate) fn surface(&self) -> &UiSurface {
        &self.surface
    }

    /// 借用已登记的资产与表面关联索引；读取此索引不会加载资产。
    /// 返回引用借用管理器中的索引，生命周期随本次 `self` 借用。
    pub(crate) fn asset_surface_index(&self) -> &UiAssetSurfaceIndex {
        &self.asset_surface_index
    }

    /// 为当前输入管理器追加节点指针回调，供之后的表面路由调用。
    ///
    /// # 参数与生命周期
    /// `node_id`、`kind` 是注册键；本入口不查询树，也不验证节点是否存在。
    /// `handler` 须适用于任意一次路由借用，捕获满足 `Send + Sync + 'static`；
    /// 闭包的静态约束不会延长上下文借用。成功切换夹具后，新输入管理器不沿用旧回调。
    ///
    /// # 参考
    /// [`Self::load_builtin_fixture`] 是表面和输入管理器的切换边界。
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

    /// 为指定阶段追加指针回调；参数与借用约束同 [`Self::register_pointer_handler`]。
    /// `phase` 原样交给底层注册表，注册一个阶段键不保证路由必然到达该阶段。
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

    /// 为节点及导航种类追加回调，供当前表面生成的导航路由调用。
    /// `node_id`、`kind` 只作为注册键；本入口不检查目标存在性。
    /// 回调借用当前导航上下文；捕获、路由借用和夹具切换约束同
    /// [`Self::register_pointer_handler`]，返回的决议交给表面继续处理。
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

    /// 处理一个规范化事件，再消费本轮表面脏状态，供后续输入使用更新后的表面。
    ///
    /// # 返回与错误
    /// 成功返回本次分发结果；分发或后续重建失败则直接返回错误。
    /// 本入口不恢复错误前已发生的状态修改，不能据返回错误推断事件毫无副作用。
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

    /// 按迭代顺序处理规范化输入，逐项沿用单事件的分发和重建顺序。
    ///
    /// # 返回与错误
    /// 全部成功才返回有序结果向量；首错记录原输入的零基索引，不交付成功前缀向量。
    /// 前序事件状态已生效；失败项也可能在后续重建报错前改变状态，均不在此回滚。
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

    /// 逐项规范化平台事件后处理；首错索引仍对应原平台事件的迭代位置。
    /// 结果交付和已提交状态的约束同 [`Self::dispatch_input_batch`]。
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

    /// 处理窗口泵事件，并在成功分发后保存创建或尺寸变化事件中的逻辑尺寸。
    /// 本轮脏表面重建使用更新后的尺寸；其他事件不改此处的尺寸。
    /// 分发失败时不保存候选尺寸，后续重建失败则不会还原已经保存的尺寸或先前修改。
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

    /// 经同步 ABI 适配再处理单个事件；适配错误与树处理错误由测试错误类型区分。
    /// 原始载荷约束见同模块窗口适配 helper：非空字节须由调用方保持有效且不变，
    /// 直到同步转换返回；按值传递事件载体不会取得其载荷内存的所有权。
    pub(crate) fn dispatch_runtime_event(
        &mut self,
        context: &UiRuntimeEventAdapterContext,
        event: ZrRuntimeEventV1,
    ) -> Result<UiInputDispatchResult, RuntimeUiManagerError> {
        super::window_event::dispatch_runtime_event(self, context, event)
    }

    /// 顺序适配并提交 ABI 事件；单事件载荷约束逐项适用，同一 `context` 用于整批。
    /// 首错返回原事件索引，前缀状态保留；只有全部成功才交付结果向量。
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

    /// 登记已成功构建的文档与节点资源引用，供反向查找表面使用。
    /// 此步骤只更新关联索引，不重新加载引用资产，也不转移资产所有权。
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

/// 将窗口逻辑尺寸逐轴四舍五入并至少取 1；非正或非有限的轴也回退为 1。
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
