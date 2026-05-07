use zircon_runtime_interface::ui::surface::{
    UiDebugOverlayPrimitive, UiDebugOverlayPrimitiveKind, UiRenderVisualizerOverlay,
    UiRenderVisualizerOverlayKind, UiSurfaceDebugSnapshot,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
        match primitive.kind {
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

    pub(crate) fn primitives_from_snapshot(
        self,
        snapshot: &UiSurfaceDebugSnapshot,
    ) -> Vec<UiDebugOverlayPrimitive> {
        let mut primitives = snapshot
            .overlay_primitives
            .iter()
            .filter(|primitive| self.allows(primitive))
            .cloned()
            .collect::<Vec<_>>();
        primitives.extend(
            snapshot
                .render_batches
                .visualizer
                .overlays
                .iter()
                .filter_map(|overlay| self.primitive_from_visualizer_overlay(overlay)),
        );
        if self.damage
            && !primitives
                .iter()
                .any(|primitive| primitive.kind == UiDebugOverlayPrimitiveKind::DamageRegion)
        {
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
        let kind = match overlay.kind {
            UiRenderVisualizerOverlayKind::Wireframe => UiDebugOverlayPrimitiveKind::Wireframe,
            UiRenderVisualizerOverlayKind::ClipScissor => UiDebugOverlayPrimitiveKind::ClipFrame,
            UiRenderVisualizerOverlayKind::BatchBounds => {
                UiDebugOverlayPrimitiveKind::MaterialBatchBounds
            }
            UiRenderVisualizerOverlayKind::OverdrawHeat => {
                UiDebugOverlayPrimitiveKind::OverdrawCell
            }
            UiRenderVisualizerOverlayKind::TextGlyphBounds => {
                UiDebugOverlayPrimitiveKind::TextGlyphBounds
            }
            UiRenderVisualizerOverlayKind::TextBaseline => {
                UiDebugOverlayPrimitiveKind::TextBaseline
            }
            UiRenderVisualizerOverlayKind::ResourceAtlas => {
                UiDebugOverlayPrimitiveKind::ResourceAtlas
            }
        };
        let primitive = UiDebugOverlayPrimitive {
            kind,
            node_id: overlay.node_id,
            frame: overlay.frame,
            label: visualizer_overlay_label(overlay),
            severity: None,
        };
        self.allows(&primitive).then_some(primitive)
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
