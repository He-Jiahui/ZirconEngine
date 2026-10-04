use std::{collections::BTreeMap, sync::Arc};

use zircon_runtime_interface::ui::{
    binding::UiEventKind,
    layout::{UiFrame, UiSize},
    surface::UiSurfaceFrame,
};

use crate::ui::binding::EditorUiBinding;
use crate::ui::retained_host::callback_dispatch::constants::BUILTIN_VIEWPORT_TOOLBAR_DOCUMENT_ID;
use crate::ui::template_runtime::{
    EditorUiHostRuntime, RetainedUiHostProjection, RetainedUiProjection,
};

#[cfg(test)]
use super::super::projection_support::load_builtin_runtime;
use super::super::projection_support::project_builtin_document_with_runtime;
use super::error::BuiltinViewportToolbarTemplateBridgeError;
use super::host_projection::{
    build_builtin_viewport_toolbar_surface, project_builtin_viewport_toolbar_host_projection,
    rebuild_builtin_viewport_toolbar_surface,
};
use super::surface_frame_cache::{ViewportToolbarSurfaceFrameCache, SURFACE_FRAME_CACHE_CAPACITY};
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::primitives::ModelRc;
use crate::ui::retained_host::ui::to_host_contract_workbench_window_nodes_with_previous_at_mount_and_scale;

// 视口工具栏保留模板投影和表面缓存，宿主按控制 ID 读取帧并按事件种类派发动作。
pub(crate) struct BuiltinViewportToolbarTemplateBridge {
    runtime: Arc<EditorUiHostRuntime>,
    projection: RetainedUiProjection,
    bindings_by_control: BTreeMap<String, BTreeMap<UiEventKind, EditorUiBinding>>,
    surface: zircon_runtime::ui::surface::UiSurface,
    host_projection: RetainedUiHostProjection,
    surface_frame_cache: ViewportToolbarSurfaceFrameCache,
    scale_factor: f32,
    theme_generation: u64,
    enter_play_enabled: bool,
    exit_play_enabled: bool,
    is_playing: bool,
    paint_layouts: BTreeMap<(u32, u32, u32, u64), ModelRc<TemplatePaneNodeData>>,
    #[cfg(test)]
    layout_recompute_count: usize,
}

impl BuiltinViewportToolbarTemplateBridge {
    #[cfg(test)]
    pub(crate) fn new() -> Result<Self, BuiltinViewportToolbarTemplateBridgeError> {
        let runtime = Arc::new(load_builtin_runtime()?);
        Self::new_with_runtime(runtime)
    }

    pub(crate) fn new_with_runtime(
        runtime: Arc<EditorUiHostRuntime>,
    ) -> Result<Self, BuiltinViewportToolbarTemplateBridgeError> {
        let projection =
            project_builtin_document_with_runtime(&runtime, BUILTIN_VIEWPORT_TOOLBAR_DOCUMENT_ID)?;
        let mut bindings_by_control =
            BTreeMap::<String, BTreeMap<UiEventKind, EditorUiBinding>>::new();
        for projected_binding in &projection.bindings {
            let path = projected_binding.binding.path();
            bindings_by_control
                .entry(path.control_id.clone())
                .or_default()
                .insert(path.event_kind, projected_binding.binding.clone());
        }
        let surface =
            build_builtin_viewport_toolbar_surface(runtime.as_ref(), UiSize::new(1280.0, 28.0))?;
        let host_projection = project_builtin_viewport_toolbar_host_projection(
            runtime.as_ref(),
            &projection,
            &surface,
        )?;
        let paint_nodes = to_host_contract_workbench_window_nodes_with_previous_at_mount_and_scale(
            Some(&host_projection),
            None,
            None,
            1.0,
        );
        let paint_layouts = BTreeMap::from([(
            (
                1280.0_f32.to_bits(),
                28.0_f32.to_bits(),
                1.0_f32.to_bits(),
                0,
            ),
            paint_nodes,
        )]);
        Ok(Self {
            runtime,
            projection,
            bindings_by_control,
            surface,
            host_projection,
            surface_frame_cache: ViewportToolbarSurfaceFrameCache::default(),
            paint_layouts,
            scale_factor: 1.0,
            theme_generation: 0,
            enter_play_enabled: false,
            exit_play_enabled: false,
            is_playing: false,
            #[cfg(test)]
            layout_recompute_count: 0,
        })
    }

    pub(crate) fn recompute_layout(
        &mut self,
        surface_size: UiSize,
    ) -> Result<(), BuiltinViewportToolbarTemplateBridgeError> {
        rebuild_builtin_viewport_toolbar_surface(
            &mut self.surface,
            UiSize::new(
                surface_size.width / self.scale_factor,
                surface_size.height / self.scale_factor,
            ),
        )?;
        self.host_projection = project_builtin_viewport_toolbar_host_projection(
            self.runtime.as_ref(),
            &self.projection,
            &self.surface,
        )?;
        self.retain_paint_layout(surface_size);
        #[cfg(test)]
        {
            self.layout_recompute_count = self.layout_recompute_count.saturating_add(1);
        }
        Ok(())
    }

    fn retain_paint_layout(&mut self, size: UiSize) {
        let key = (
            size.width.to_bits(),
            size.height.to_bits(),
            self.scale_factor.to_bits(),
            self.theme_generation,
        );
        self.paint_layouts.insert(
            key,
            to_host_contract_workbench_window_nodes_with_previous_at_mount_and_scale(
                Some(&self.host_projection),
                None,
                None,
                self.scale_factor,
            ),
        );
        if self.paint_layouts.len() > SURFACE_FRAME_CACHE_CAPACITY {
            if let Some(oldest_key) = self
                .paint_layouts
                .keys()
                .copied()
                .find(|candidate| *candidate != key)
            {
                self.paint_layouts.remove(&oldest_key);
            }
        }
    }

    /// A cached hit layout may be revisited after another leaf changed the bridge's
    /// current width. Keep its complete visual projection instead of using that leaf.
    pub(crate) fn paint_nodes_for_size(
        &mut self,
        size: UiSize,
    ) -> Result<ModelRc<TemplatePaneNodeData>, BuiltinViewportToolbarTemplateBridgeError> {
        let key = (
            size.width.to_bits(),
            size.height.to_bits(),
            self.scale_factor.to_bits(),
            self.theme_generation,
        );
        if !self.paint_layouts.contains_key(&key) {
            self.recompute_layout(size)?;
        }
        Ok(self.paint_layouts.get(&key).cloned().unwrap_or_default())
    }

    pub(crate) fn admit_layout_context(&mut self, scale: f32, theme_generation: u64) {
        let scale = if scale.is_finite() && scale > 0.0 {
            scale
        } else {
            1.0
        };
        if self.scale_factor.to_bits() != scale.to_bits()
            || self.theme_generation != theme_generation
        {
            self.scale_factor = scale;
            self.theme_generation = theme_generation;
            self.paint_layouts.clear();
            self.surface_frame_cache = ViewportToolbarSurfaceFrameCache::default();
        }
    }

    pub(crate) fn scale_factor(&self) -> f32 {
        self.scale_factor
    }
    pub(crate) fn set_play_admission(&mut self, enter: bool, exit: bool, is_playing: bool) {
        self.enter_play_enabled = enter;
        self.exit_play_enabled = exit;
        self.is_playing = is_playing;
    }
    pub(crate) fn play_admission(&self) -> (bool, bool, bool) {
        (
            self.enter_play_enabled,
            self.exit_play_enabled,
            self.is_playing,
        )
    }

    fn physical_hit_projection(&self) -> RetainedUiHostProjection {
        let mut projection = self.host_projection.clone();
        for node in &mut projection.nodes {
            node.frame.x *= self.scale_factor;
            node.frame.y *= self.scale_factor;
            node.frame.width *= self.scale_factor;
            node.frame.height *= self.scale_factor;
            if let Some(clip) = &mut node.clip_frame {
                clip.x *= self.scale_factor;
                clip.y *= self.scale_factor;
                clip.width *= self.scale_factor;
                clip.height *= self.scale_factor;
            }
        }
        projection
    }

    pub(crate) fn binding_for_control(
        &self,
        control_id: &str,
        event_kind: UiEventKind,
    ) -> Option<&EditorUiBinding> {
        self.bindings_by_control
            .get(control_id)
            .and_then(|bindings| bindings.get(&event_kind))
    }

    #[cfg(test)]
    pub(crate) fn layout_recompute_count(&self) -> usize {
        self.layout_recompute_count
    }

    pub(crate) fn control_frame_for_control(&self, control_id: &str) -> Option<UiFrame> {
        self.host_projection
            .node_by_control_id(control_id)
            .map(|node| node.frame)
    }

    pub(crate) fn surface_frame_for_projection_controls<F>(
        &mut self,
        surface_key: &str,
        surface_size: UiSize,
        hit_control_id: F,
    ) -> Arc<UiSurfaceFrame>
    where
        F: FnMut(&str) -> Option<String>,
    {
        let projection = self.physical_hit_projection();
        self.surface_frame_cache.resolve(
            &projection,
            surface_key,
            surface_size,
            None,
            hit_control_id,
        )
    }

    pub(crate) fn surface_frame_for_projection_controls_with_hit_route_key<F>(
        &mut self,
        surface_key: &str,
        surface_size: UiSize,
        hit_route_key: &[&str],
        hit_control_id: F,
    ) -> Arc<UiSurfaceFrame>
    where
        F: FnMut(&str) -> Option<String>,
    {
        let projection = self.physical_hit_projection();
        self.surface_frame_cache.resolve(
            &projection,
            surface_key,
            surface_size,
            Some(hit_route_key),
            hit_control_id,
        )
    }

    pub(crate) fn surface_frame_from_cached_layout_for_projection_controls<F>(
        &mut self,
        surface_key: &str,
        surface_size: UiSize,
        hit_control_id: F,
    ) -> Option<Arc<UiSurfaceFrame>>
    where
        F: FnMut(&str) -> Option<String>,
    {
        self.surface_frame_cache.resolve_if_layout_matches(
            surface_key,
            surface_size,
            None,
            hit_control_id,
        )
    }

    // 命中路由已知时仅复用相同布局的帧；返回空值要求宿主走完整投影路径。
    pub(crate) fn surface_frame_from_cached_layout_for_projection_controls_with_hit_route_key<F>(
        &mut self,
        surface_key: &str,
        surface_size: UiSize,
        hit_route_key: &[&str],
        hit_control_id: F,
    ) -> Option<Arc<UiSurfaceFrame>>
    where
        F: FnMut(&str) -> Option<String>,
    {
        self.surface_frame_cache.resolve_if_layout_matches(
            surface_key,
            surface_size,
            Some(hit_route_key),
            hit_control_id,
        )
    }

    #[cfg(test)]
    pub(crate) fn hit_control_projection_count(&self) -> usize {
        self.surface_frame_cache.hit_control_projection_count()
    }
}
