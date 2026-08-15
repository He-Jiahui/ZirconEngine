use std::{cell::RefCell, sync::Arc};

use crate::ui::retained_host::ui_perf::{record_current_ui_perf_counter, UiPerfCounter};
use zircon_runtime_interface::ui::template::UiAssetDocument;

use super::super::{
    source_sync::{
        build_source_outline_index, source_byte_offset_for_line, UiAssetSourceOutlineCache,
    },
    ui_asset_editor_session::{UiAssetEditorSession, UiAssetSourceCursorAnchor},
};

pub(super) struct UiAssetSourceOutlineInitialState {
    pub(super) cursor_byte_offset: usize,
    pub(super) source_outline_cache: RefCell<UiAssetSourceOutlineCache>,
    pub(super) last_valid_source_outline_cache: RefCell<UiAssetSourceOutlineCache>,
}

pub(super) fn initial_source_outline_state(
    document: &UiAssetDocument,
    source: &str,
    cursor_anchor: Option<&UiAssetSourceCursorAnchor>,
) -> UiAssetSourceOutlineInitialState {
    let outline = Arc::new(build_source_outline_index(document, source));
    record_current_ui_perf_counter(UiPerfCounter::AssetEditorPaneSourceBuildCount, 1.0);
    let cursor_byte_offset = cursor_anchor
        .and_then(|anchor| {
            outline
                .entry_for_node(&anchor.node_id)
                .map(|entry| source_byte_offset_for_line(source, entry.line as usize))
        })
        .unwrap_or(0);

    UiAssetSourceOutlineInitialState {
        cursor_byte_offset,
        source_outline_cache: RefCell::new(UiAssetSourceOutlineCache::from_built(
            0,
            Arc::clone(&outline),
        )),
        last_valid_source_outline_cache: RefCell::new(UiAssetSourceOutlineCache::new(0, outline)),
    }
}

pub(super) fn refresh_valid_source_outline_caches(session: &mut UiAssetEditorSession) {
    session.last_valid_source_generation = session.last_valid_source_generation.wrapping_add(1);
    let source_revision = session.source_buffer.revision();
    let outline = Arc::new(build_source_outline_index(
        &session.last_valid_document,
        session.source_buffer.text(),
    ));
    record_current_ui_perf_counter(UiPerfCounter::AssetEditorPaneSourceBuildCount, 1.0);
    session
        .source_outline_cache
        .borrow_mut()
        .replace_shared_built(source_revision, Arc::clone(&outline));
    session
        .last_valid_source_outline_cache
        .borrow_mut()
        .replace_shared(session.last_valid_source_generation, outline);
}
