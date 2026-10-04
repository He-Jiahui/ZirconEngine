//! 诊断pane只消费当前surface快照；开关裁剪已产生的叠加并按需补damage区域。
use zircon_runtime_interface::ui::surface::{
    UiDebugOverlayPrimitive, UiDebugOverlayPrimitiveKind, UiRenderVisualizerOverlay,
    UiRenderVisualizerOverlayKind, UiSurfaceDebugSnapshot,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// 诊断overlay的显示选择；默认开启，但不改变surface中的原始调试数据。
pub(crate) struct EditorUiDebugReflectorOverlayState {
    pub selected_frame: bool,
    pub clip_frame: bool,
    pub wireframe: bool,
    pub hit_grid: bool,
    pub hit_path: bool,
    pub rejected_bounds: bool,
    pub overdraw: bool,
    pub material_batches: bool,
    pub text_debug: bool,
    pub resource_atlas: bool,
    pub damage: bool,
}

impl Default for EditorUiDebugReflectorOverlayState {
    fn default() -> Self {
        Self {
            selected_frame: true,
            clip_frame: true,
            wireframe: true,
            hit_grid: true,
            hit_path: true,
            rejected_bounds: true,
            overdraw: true,
            material_batches: true,
            text_debug: true,
            resource_atlas: true,
            damage: true,
        }
    }
}

impl EditorUiDebugReflectorOverlayState {
    pub(crate) fn allows(self, primitive: &UiDebugOverlayPrimitive) -> bool {
        self.allows_kind(primitive.kind)
    }

    fn allows_kind(self, kind: UiDebugOverlayPrimitiveKind) -> bool {
        match kind {
            UiDebugOverlayPrimitiveKind::SelectedFrame => self.selected_frame,
            UiDebugOverlayPrimitiveKind::ClipFrame => self.clip_frame,
            UiDebugOverlayPrimitiveKind::Wireframe => self.wireframe,
            UiDebugOverlayPrimitiveKind::HitCell => self.hit_grid,
            UiDebugOverlayPrimitiveKind::HitPath => self.hit_path,
            UiDebugOverlayPrimitiveKind::RejectedBounds => self.rejected_bounds,
            UiDebugOverlayPrimitiveKind::OverdrawCell => self.overdraw,
            UiDebugOverlayPrimitiveKind::MaterialBatchBounds => self.material_batches,
            UiDebugOverlayPrimitiveKind::TextGlyphBounds
            | UiDebugOverlayPrimitiveKind::TextBaseline => self.text_debug,
            UiDebugOverlayPrimitiveKind::ResourceAtlas => self.resource_atlas,
            UiDebugOverlayPrimitiveKind::DamageRegion => self.damage,
        }
    }

    /// 将共享叠加与可视化叠加按来源顺序投影；仅在缺失共享项时补damage区域。
    pub(crate) fn primitives_from_snapshot(
        self,
        snapshot: &UiSurfaceDebugSnapshot,
    ) -> Vec<UiDebugOverlayPrimitive> {
        let shared_count = snapshot
            .overlay_primitives
            .iter()
            .filter(|primitive| self.allows(primitive))
            .count();
        let visualizer_count = snapshot
            .render_batches
            .visualizer
            .overlays
            .iter()
            .filter(|overlay| self.allows_visualizer_overlay(overlay))
            .count();
        let synthesize_damage = self.damage
            && snapshot.damage.damage_region.is_some()
            && !snapshot
                .overlay_primitives
                .iter()
                .any(|primitive| primitive.kind == UiDebugOverlayPrimitiveKind::DamageRegion);
        let mut primitives =
            Vec::with_capacity(shared_count + visualizer_count + usize::from(synthesize_damage));
        primitives.extend(
            snapshot
                .overlay_primitives
                .iter()
                .filter(|primitive| self.allows(primitive))
                .cloned(),
        );
        primitives.extend(
            snapshot
                .render_batches
                .visualizer
                .overlays
                .iter()
                .filter_map(|overlay| self.primitive_from_visualizer_overlay(overlay)),
        );
        if synthesize_damage {
            if let Some(frame) = snapshot.damage.damage_region {
                primitives.push(UiDebugOverlayPrimitive {
                    kind: UiDebugOverlayPrimitiveKind::DamageRegion,
                    node_id: None,
                    frame,
                    label: Some("damage".to_string()),
                    severity: Some("warning".to_string()),
                });
            }
        }
        primitives
    }

    fn primitive_from_visualizer_overlay(
        self,
        overlay: &UiRenderVisualizerOverlay,
    ) -> Option<UiDebugOverlayPrimitive> {
        let kind = visualizer_overlay_kind(overlay.kind);
        self.allows_kind(kind).then(|| UiDebugOverlayPrimitive {
            kind,
            node_id: overlay.node_id,
            frame: overlay.frame,
            label: visualizer_overlay_label(overlay),
            severity: None,
        })
    }

    fn allows_visualizer_overlay(self, overlay: &UiRenderVisualizerOverlay) -> bool {
        self.allows_kind(visualizer_overlay_kind(overlay.kind))
    }
}

fn visualizer_overlay_kind(kind: UiRenderVisualizerOverlayKind) -> UiDebugOverlayPrimitiveKind {
    match kind {
        UiRenderVisualizerOverlayKind::Wireframe => UiDebugOverlayPrimitiveKind::Wireframe,
        UiRenderVisualizerOverlayKind::ClipScissor => UiDebugOverlayPrimitiveKind::ClipFrame,
        UiRenderVisualizerOverlayKind::BatchBounds => {
            UiDebugOverlayPrimitiveKind::MaterialBatchBounds
        }
        UiRenderVisualizerOverlayKind::OverdrawHeat => UiDebugOverlayPrimitiveKind::OverdrawCell,
        UiRenderVisualizerOverlayKind::TextGlyphBounds => {
            UiDebugOverlayPrimitiveKind::TextGlyphBounds
        }
        UiRenderVisualizerOverlayKind::TextBaseline => UiDebugOverlayPrimitiveKind::TextBaseline,
        UiRenderVisualizerOverlayKind::ResourceAtlas => UiDebugOverlayPrimitiveKind::ResourceAtlas,
    }
}

fn visualizer_overlay_label(overlay: &UiRenderVisualizerOverlay) -> Option<String> {
    overlay.label.clone().or_else(|| {
        overlay
            .batch_index
            .map(|batch_index| format!("batch:{batch_index}"))
            .or_else(|| {
                overlay
                    .paint_index
                    .map(|paint_index| format!("paint:{paint_index}"))
            })
    })
}

#[cfg(test)]
#[path = "tests/overlay.rs"]
mod tests;
