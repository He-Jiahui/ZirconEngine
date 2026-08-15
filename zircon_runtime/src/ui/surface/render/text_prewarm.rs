use crate::core::runtime::tasks::{TaskPool, TaskPools};
use crate::text::TextDocumentKey;
use crate::ui::text::{
    resolve_text_layout, UiTextLayoutRequest, UiTextMeasureCache, UiTextShapePrewarmRequest,
    UiTextViewport,
};
use zircon_runtime_interface::ui::layout::UiFrame;
use zircon_runtime_interface::ui::surface::{UiEditableTextState, UiRenderCommand};
#[cfg(feature = "profiling")]
mod profile;
#[cfg(feature = "profiling")]
pub(super) use profile::{
    record_compiled_rich_text_cache_profile, record_text_extract_profile,
    TextFontHandleFrameProfile,
};
const UI_TEXT_SHAPE_PREWARM_CHUNK_SIZE: usize = 8;
pub(super) const UI_TEXT_OWNER_PREWARM_OVERLAP_MIN_REQUESTS: usize = 8;
/// Owner layout inputs retained outside renderer-facing DTOs, ordered by command index.
/// They preserve document identity, viewport, and editable state through prewarm and resolution.
#[derive(Default)]
pub(super) struct PendingOwnerTextLayouts {
    entries: Vec<PendingOwnerTextLayout>,
}

impl PendingOwnerTextLayouts {
    pub(super) fn push(
        &mut self,
        command_index: usize,
        document_key: TextDocumentKey,
        viewport: Option<UiTextViewport>,
        editable: Option<UiEditableTextState>,
    ) {
        self.entries.push(PendingOwnerTextLayout {
            command_index,
            document_key,
            viewport,
            editable,
        });
    }

    #[cfg(feature = "profiling")]
    pub(super) fn len(&self) -> usize {
        self.entries.len()
    }
}

struct PendingOwnerTextLayout {
    command_index: usize,
    document_key: TextDocumentKey,
    viewport: Option<UiTextViewport>,
    editable: Option<UiEditableTextState>,
}

pub(super) fn prewarm_render_command_text(
    commands: &[UiRenderCommand],
    pending_owner_text_layouts: &PendingOwnerTextLayouts,
    text_measure_cache: &mut UiTextMeasureCache,
) {
    prewarm_render_command_text_after_owner_overlap(
        commands,
        pending_owner_text_layouts,
        text_measure_cache,
        false,
    );
}

pub(super) fn prewarm_render_command_text_after_owner_overlap(
    commands: &[UiRenderCommand],
    pending_owner_text_layouts: &PendingOwnerTextLayouts,
    text_measure_cache: &mut UiTextMeasureCache,
    owner_text_already_prewarmed: bool,
) {
    crate::profile_scope!("runtime", "ui_text.prewarm", "render_command_text");
    let mut pending_index = 0;
    let requests = commands
        .iter()
        .enumerate()
        .filter_map(|(command_index, command)| {
            let pending_owner = pending_owner_text_layouts
                .entries
                .get(pending_index)
                .filter(|pending| pending.command_index == command_index);
            if pending_owner.is_some() {
                pending_index += 1;
            }
            if owner_text_already_prewarmed && pending_owner.is_some() {
                return None;
            }
            if !command_text_can_use_shape_prewarm(command) {
                return None;
            }
            let viewport_owner = pending_owner
                .and_then(|pending| pending_owner_text_request(command, pending))
                .is_some_and(|request| {
                    text_measure_cache.viewport_selects_partial_plain_text(&request)
                });
            // Only a true hard-line subset may defer the full-source batch. The layout query
            // shares its metrics sample and hard-line index with the resolving request.
            if viewport_owner {
                return None;
            }
            UiTextShapePrewarmRequest::from_layout_source(
                command.text.as_deref()?,
                command.style.clone(),
            )
        })
        .collect::<Vec<_>>();
    debug_assert_eq!(pending_index, pending_owner_text_layouts.entries.len());
    if owner_text_already_prewarmed && requests.is_empty() {
        return;
    }
    prewarm_text_requests_and_record(&requests, text_measure_cache);
}

pub(super) fn prewarm_owner_text_requests(
    requests: &[UiTextShapePrewarmRequest],
    text_measure_cache: &mut UiTextMeasureCache,
) {
    crate::profile_scope!("runtime", "ui_text.prewarm", "owner_text_overlap");
    prewarm_text_requests_and_record(requests, text_measure_cache);
}

fn prewarm_text_requests_and_record(
    requests: &[UiTextShapePrewarmRequest],
    text_measure_cache: &mut UiTextMeasureCache,
) {
    let report = prewarm_text_requests(requests, text_measure_cache);
    #[cfg(feature = "profiling")]
    profile::record_text_prewarm_profile(report);
    #[cfg(not(feature = "profiling"))]
    let _ = report;
}

fn prewarm_text_requests(
    requests: &[UiTextShapePrewarmRequest],
    text_measure_cache: &mut UiTextMeasureCache,
) -> crate::text::parallel::shape_pool::TextParallelShapeBatchReport {
    if requests.is_empty() {
        Default::default()
    } else {
        let pool = ui_text_shape_prewarm_pool();
        text_measure_cache.prewarm_horizontal_paragraphs(
            &pool,
            requests,
            UI_TEXT_SHAPE_PREWARM_CHUNK_SIZE,
        )
    }
}

pub(super) fn resolve_missing_render_command_text_layouts(
    commands: &mut [UiRenderCommand],
    pending_owner_text_layouts: &PendingOwnerTextLayouts,
    mut text_measure_cache: Option<&mut UiTextMeasureCache>,
) {
    #[cfg(feature = "profiling")]
    let shaped_run_cache_before = text_measure_cache
        .as_deref()
        .map(UiTextMeasureCache::frame_shaped_run_report);
    #[cfg(feature = "profiling")]
    let uncached_document_resolves_before = text_measure_cache
        .as_deref()
        .map(UiTextMeasureCache::frame_uncached_document_resolve_count)
        .unwrap_or_default();
    {
        crate::profile_scope!("runtime", "ui_text.layout_resolve", "render_command_text");
        let mut pending_index = 0;
        for (command_index, command) in commands.iter_mut().enumerate() {
            if let Some(pending) = pending_owner_text_layouts.entries.get(pending_index) {
                if pending.command_index == command_index {
                    resolve_pending_owner_text_layout(
                        command,
                        pending,
                        text_measure_cache.as_deref_mut(),
                    );
                    pending_index += 1;
                    continue;
                }
            }
            if !command_text_needs_layout(command) {
                continue;
            }
            let Some(text) = command.text.as_deref() else {
                continue;
            };
            let request =
                UiTextLayoutRequest::new(text, &command.style, command.frame, command.clip_frame);
            let layout = match text_measure_cache.as_deref_mut() {
                Some(cache) => cache.resolve_or_shape(&request).layout,
                None => resolve_text_layout(&request).layout,
            };
            command.text_layout = Some(layout);
        }
        debug_assert_eq!(pending_index, pending_owner_text_layouts.entries.len());
    }
    #[cfg(feature = "profiling")]
    if let (Some(cache), Some(shaped_run_cache_before)) =
        (text_measure_cache.as_deref(), shaped_run_cache_before)
    {
        profile::record_text_layout_resolve_profile(
            cache,
            shaped_run_cache_before,
            cache
                .frame_uncached_document_resolve_count()
                .saturating_sub(uncached_document_resolves_before),
        );
    }
}

fn resolve_pending_owner_text_layout(
    command: &mut UiRenderCommand,
    pending: &PendingOwnerTextLayout,
    text_measure_cache: Option<&mut UiTextMeasureCache>,
) {
    let Some(request) = pending_owner_text_request(command, pending) else {
        return;
    };
    let mut layout = match text_measure_cache {
        Some(cache) => cache.resolve_or_shape(&request).layout,
        None => resolve_text_layout(&request).layout,
    };
    layout.editable = pending.editable.clone();
    command.text_layout = Some(layout);
}

fn pending_owner_text_request<'a>(
    command: &'a UiRenderCommand,
    pending: &PendingOwnerTextLayout,
) -> Option<UiTextLayoutRequest<'a>> {
    let text = command.text.as_deref()?;
    let mut request =
        UiTextLayoutRequest::new(text, &command.style, command.frame, command.clip_frame)
            .with_document_key(pending.document_key);
    if let Some(viewport) = pending.viewport {
        request = request.with_viewport(viewport);
    }
    Some(request)
}

fn command_text_can_use_shape_prewarm(command: &UiRenderCommand) -> bool {
    command_text_needs_layout(command)
}

fn command_text_needs_layout(command: &UiRenderCommand) -> bool {
    command.text_layout.is_none()
        && command
            .text
            .as_ref()
            .is_some_and(|text| !text.trim().is_empty())
        && valid_text_frame(command.frame)
}

fn valid_text_frame(frame: UiFrame) -> bool {
    frame.width.is_finite() && frame.height.is_finite() && frame.width > 0.0 && frame.height > 0.0
}

pub(super) fn ui_text_shape_prewarm_pool() -> TaskPool {
    TaskPools::process_default().compute().clone()
}

#[cfg(all(test, feature = "profiling"))]
#[path = "text_prewarm/tests/profile.rs"]
mod profile_test;

#[cfg(test)]
#[path = "text_prewarm/tests.rs"]
mod tests;
