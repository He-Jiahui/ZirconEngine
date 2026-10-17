use super::error;
use crate::asset::runtime::browser::BrowserAssetSource;
use crate::asset::{
    AssetKind, RuntimeAssetManager, UiV2ComponentAsset, UiV2StyleAsset, UiV2ViewAsset,
};
use crate::core::framework::input::{InputButton, InputEvent, TouchPhase};
use crate::core::framework::render::{
    UiRenderNodeIdProjection, UiRenderSubmission, UiRenderSubmissionSegment,
};
use crate::core::math::UVec2;
use crate::text::{
    font::{prepare_runtime_font_asset_admission, RuntimeFontAssetClaimScope},
    TextRuntimeContext,
};
use crate::ui::{
    dispatch::UiInputManager,
    surface::UiSurface,
    v2::{UiV2PrototypeStoreBuilder, UiV2SurfaceBuilder},
};
use std::sync::Arc;
use zircon_runtime_interface::ui::{
    dispatch::{
        UiInputEvent, UiInputEventMetadata, UiInputSequence, UiInputTimestamp,
        UiKeyboardInputEvent, UiKeyboardInputState, UiPointerEvent, UiPointerId,
        UiPointerInputEvent, UiPointerSource,
    },
    event_ui::UiTreeId,
    layout::{UiPoint, UiSize},
    surface::{UiPointerButton, UiPointerEventKind},
    window::{UiWindowEvent, UiWindowInputPumpEvent},
};
struct Surface {
    view: UiSurface,
    input: UiInputManager,
}
pub(super) struct BrowserUi {
    surfaces: Vec<Surface>,
    _fonts: Option<RuntimeFontAssetClaimScope>,
    generations: Vec<u64>,
    size: Option<UVec2>,
    cached: Option<Arc<UiRenderSubmission>>,
    sequence: u64,
    cursor: UiPoint,
}
impl BrowserUi {
    pub(super) fn load(
        source: &BrowserAssetSource,
        text: &TextRuntimeContext,
        manager: &RuntimeAssetManager,
    ) -> Result<Self, String> {
        let mut surfaces = Vec::new();
        let mut fonts = None;
        if !source.ui_roots().is_empty() {
            let resources = source.resource_manager();
            let registry = resources.registry();
            let mut builder = UiV2PrototypeStoreBuilder::new();
            for id in source.startup_asset_ids() {
                let record = registry
                    .get(id)
                    .ok_or_else(|| "UI resource is missing".to_string())?;
                if !matches!(
                    record.kind,
                    AssetKind::UiLayout | AssetKind::UiWidget | AssetKind::UiStyle
                ) {
                    continue;
                }
                let payload = resources
                    .get_untyped(id)
                    .ok_or_else(|| "UI dependency is not resident".to_string())?
                    .into_any_arc();
                let document = if let Some(asset) = payload.downcast_ref::<UiV2ViewAsset>() {
                    &asset.document
                } else if let Some(asset) = payload.downcast_ref::<UiV2ComponentAsset>() {
                    &asset.document
                } else if let Some(asset) = payload.downcast_ref::<UiV2StyleAsset>() {
                    &asset.document
                } else {
                    return Err("UI catalog kind does not match its document".into());
                };
                let uri = record.primary_locator.to_string();
                let _ = builder.insert_with_aliases_and_source_path(
                    document.clone(),
                    [uri.clone()],
                    uri,
                );
            }
            let roots = source
                .ui_roots()
                .iter()
                .map(|id| {
                    registry
                        .get(*id)
                        .expect("validated UI root")
                        .primary_locator
                        .to_string()
                })
                .collect::<Vec<_>>();
            let prototypes = builder
                .build_for_roots(roots.iter().cloned())
                .map_err(error)?;
            for (index, uri) in roots.iter().enumerate() {
                let document = prototypes
                    .get(uri)
                    .ok_or_else(|| format!("missing UI root {uri}"))?;
                let view = UiV2SurfaceBuilder::build_surface_with_prototype_store_and_text_context(
                    UiTreeId::new(format!("browser-ui-{index}")),
                    document.as_ref(),
                    &prototypes,
                    text,
                )
                .map_err(error)?;
                surfaces.push(Surface {
                    view,
                    input: UiInputManager::summary(),
                });
            }
            let mut dependencies = surfaces
                .iter()
                .flat_map(|surface| surface.view.text_font_asset_dependencies())
                .collect::<Vec<_>>();
            dependencies.sort_unstable();
            dependencies.dedup();
            let dependencies = dependencies
                .iter()
                .map(|uri| Arc::<str>::from(uri.as_str()))
                .collect::<Vec<_>>();
            let mut scope = text.font_collection().runtime_font_asset_claim_scope();
            let admissions = dependencies
                .iter()
                .cloned()
                .map(|uri| prepare_runtime_font_asset_admission(manager, uri))
                .collect();
            let transition = scope.replace_shared_claims_with_admissions(&dependencies, admissions);
            for admission in transition.admissions {
                admission.result.map_err(error)?;
            }
            fonts = Some(scope);
        }
        Ok(Self {
            surfaces,
            _fonts: fonts,
            generations: Vec::new(),
            size: None,
            cached: None,
            sequence: 0,
            cursor: UiPoint::default(),
        })
    }
    pub(super) fn input(&mut self, event: &InputEvent, timestamp_ms: f64) -> Result<(), String> {
        if self.surfaces.is_empty() {
            return Ok(());
        }
        self.sequence = self
            .sequence
            .checked_add(1)
            .ok_or_else(|| "UI input sequence exhausted".to_string())?;
        let mut metadata = UiInputEventMetadata::new(
            UiInputTimestamp::from_micros((timestamp_ms * 1000.0).min(u64::MAX as f64) as u64),
            UiInputSequence::new(self.sequence),
        );
        if matches!(event, InputEvent::FocusLost) {
            for surface in &mut self.surfaces {
                surface
                    .input
                    .dispatch_window_input_pump_event(
                        &mut surface.view,
                        UiWindowInputPumpEvent::Window(UiWindowEvent::window_focused(
                            Default::default(),
                            false,
                        )),
                    )
                    .map_err(error)?;
            }
            return Ok(());
        }
        let translated = match event {
            InputEvent::CursorMoved { x, y } => {
                self.cursor = UiPoint::new(*x, *y);
                Some(UiPointerEvent::new(UiPointerEventKind::Move, self.cursor))
            }
            InputEvent::ButtonPressed(button) | InputEvent::ButtonReleased(button) => {
                let button = match button {
                    InputButton::MouseLeft => UiPointerButton::Primary,
                    InputButton::MouseRight => UiPointerButton::Secondary,
                    InputButton::MouseMiddle => UiPointerButton::Middle,
                    _ => return Ok(()),
                };
                Some(
                    UiPointerEvent::new(
                        if matches!(event, InputEvent::ButtonPressed(_)) {
                            UiPointerEventKind::Down
                        } else {
                            UiPointerEventKind::Up
                        },
                        self.cursor,
                    )
                    .with_button(button),
                )
            }
            InputEvent::WheelScrolled { delta } => Some(
                UiPointerEvent::new(UiPointerEventKind::Scroll, self.cursor)
                    .with_scroll_delta(*delta),
            ),
            InputEvent::Touch { id, phase, x, y } => {
                metadata.pointer_id = Some(UiPointerId::new(*id));
                metadata.pointer_source = UiPointerSource::Touch;
                Some(
                    UiPointerEvent::new(
                        match phase {
                            TouchPhase::Started => UiPointerEventKind::Down,
                            TouchPhase::Moved => UiPointerEventKind::Move,
                            TouchPhase::Ended => UiPointerEventKind::Up,
                            TouchPhase::Cancelled => UiPointerEventKind::Cancel,
                        },
                        UiPoint::new(*x, *y),
                    )
                    .with_button(UiPointerButton::Primary),
                )
            }
            InputEvent::KeyboardInput {
                key_code,
                logical_key,
                text,
                pressed,
                repeat,
            } => {
                let key = logical_key.clone().unwrap_or_default();
                let keyboard = UiInputEvent::Keyboard(UiKeyboardInputEvent {
                    metadata,
                    state: if !pressed {
                        UiKeyboardInputState::Released
                    } else if *repeat {
                        UiKeyboardInputState::Repeated
                    } else {
                        UiKeyboardInputState::Pressed
                    },
                    key_code: *key_code,
                    scan_code: None,
                    physical_key: key.clone(),
                    logical_key: key,
                    text: text.clone(),
                });
                let top = self.surfaces.last_mut().expect("UI root exists");
                top.input
                    .dispatch_input_event(&mut top.view, keyboard)
                    .map_err(error)?;
                return Ok(());
            }
            _ => None,
        };
        if let Some(event) = translated {
            let top = self.surfaces.last_mut().expect("UI root exists");
            top.input
                .dispatch_input_event(
                    &mut top.view,
                    UiInputEvent::Pointer(UiPointerInputEvent {
                        metadata,
                        event,
                        precise_scroll: None,
                    }),
                )
                .map_err(error)?;
        }
        Ok(())
    }
    pub(super) fn submission(
        &mut self,
        size: UVec2,
        timestamp_ms: f64,
    ) -> Result<Option<Arc<UiRenderSubmission>>, String> {
        if self.surfaces.is_empty() {
            return Ok(None);
        }
        let timestamp =
            UiInputTimestamp::from_micros((timestamp_ms * 1000.0).min(u64::MAX as f64) as u64);
        for surface in &mut self.surfaces {
            surface
                .input
                .tick(&mut surface.view, timestamp)
                .map_err(error)?;
            surface
                .input
                .synchronize_text_document_owners(&mut surface.view);
            surface
                .view
                .rebuild_dirty(UiSize::new(size.x as f32, size.y as f32))
                .map_err(error)?;
        }
        if self.size == Some(size)
            && self.generations.len() == self.surfaces.len()
            && self
                .generations
                .iter()
                .zip(&self.surfaces)
                .all(|(generation, surface)| {
                    *generation == surface.view.invalidation_generations().render
                })
        {
            return Ok(self.cached.as_ref().map(Arc::clone));
        }
        let segments = self
            .surfaces
            .iter()
            .enumerate()
            .map(|(index, surface)| {
                UiRenderSubmissionSegment::projected(
                    surface.view.render_frame_extract(),
                    UiTreeId::new("browser-ui"),
                    UiRenderNodeIdProjection::new(((index as u64) + 1) << 48, (1_u64 << 48) - 1),
                )
            })
            .collect();
        let submission = UiRenderSubmission::from_submission_segments(segments);
        self.generations.clear();
        self.generations.extend(
            self.surfaces
                .iter()
                .map(|surface| surface.view.invalidation_generations().render),
        );
        self.size = Some(size);
        self.cached = Some(Arc::clone(&submission));
        Ok(Some(submission))
    }
}
