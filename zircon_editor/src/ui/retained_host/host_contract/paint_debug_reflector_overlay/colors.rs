use zircon_runtime_interface::ui::surface::UiDebugOverlayPrimitiveKind;

pub(in crate::ui::retained_host::host_contract) const LABEL_TEXT: [u8; 4] = [230, 236, 246, 230];

const SELECTED_FRAME: [u8; 4] = [92, 156, 255, 255];
const CLIP_FRAME: [u8; 4] = [148, 117, 255, 220];
const WIREFRAME: [u8; 4] = [64, 196, 255, 180];
const HIT_CELL: [u8; 4] = [64, 220, 142, 96];
const HIT_PATH: [u8; 4] = [64, 220, 142, 220];
const REJECTED_BOUNDS: [u8; 4] = [190, 198, 214, 120];
const OVERDRAW: [u8; 4] = [255, 167, 38, 104];
const MATERIAL_BATCH: [u8; 4] = [64, 188, 255, 96];
const TEXT_DEBUG: [u8; 4] = [186, 104, 200, 176];
const RESOURCE_ATLAS: [u8; 4] = [38, 166, 154, 148];
const DAMAGE_REGION: [u8; 4] = [255, 88, 112, 128];

pub(in crate::ui::retained_host::host_contract) fn overlay_color(
    kind: UiDebugOverlayPrimitiveKind,
) -> [u8; 4] {
    match kind {
        UiDebugOverlayPrimitiveKind::SelectedFrame => SELECTED_FRAME,
        UiDebugOverlayPrimitiveKind::ClipFrame => CLIP_FRAME,
        UiDebugOverlayPrimitiveKind::Wireframe => WIREFRAME,
        UiDebugOverlayPrimitiveKind::HitCell => HIT_CELL,
        UiDebugOverlayPrimitiveKind::HitPath => HIT_PATH,
        UiDebugOverlayPrimitiveKind::RejectedBounds => REJECTED_BOUNDS,
        UiDebugOverlayPrimitiveKind::OverdrawCell => OVERDRAW,
        UiDebugOverlayPrimitiveKind::MaterialBatchBounds => MATERIAL_BATCH,
        UiDebugOverlayPrimitiveKind::TextGlyphBounds
        | UiDebugOverlayPrimitiveKind::TextBaseline => TEXT_DEBUG,
        UiDebugOverlayPrimitiveKind::ResourceAtlas => RESOURCE_ATLAS,
        UiDebugOverlayPrimitiveKind::DamageRegion => DAMAGE_REGION,
    }
}

pub(in crate::ui::retained_host::host_contract) fn solid_border_color(
    mut color: [u8; 4],
) -> [u8; 4] {
    color[3] = color[3].saturating_add(80).max(180);
    color
}
