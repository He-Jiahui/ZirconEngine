use zircon_runtime_interface::ui::component::UiValue;

use crate::core::commands::{
    CommandEvalCtx, EditorCommandPaletteCatalog, EditorCommandPaletteMru,
    EditorCommandPaletteQueryWindow,
};
use crate::ui::binding::EditorUiEventKind;
use crate::ui::retained_host::event_bridge::UiHostEventEffects;

use super::callback_dispatch::WorkbenchCommandPaletteOpenState;
use super::{HostInvalidationMask, RetainedEditorHost};

const COMMAND_PALETTE_COMMAND_ID: &str = "editor.command.palette";
const COMMAND_PALETTE_CONTROL_ID: &str = "WorkbenchCommandPalette";
const COMMAND_PALETTE_QUERY_BINDING_ID: &str = "CommandPalette/QueryChanged";
const COMMAND_PALETTE_WINDOW_BINDING_ID: &str = "CommandPalette/WindowRequested";
const COMMAND_PALETTE_VISIBLE_ROWS: usize = 8;
const COMMAND_PALETTE_OVERSCAN_ROWS: usize = 4;

impl RetainedEditorHost {
    pub(super) fn open_workbench_command_palette(&mut self) {
        let context = self.runtime.context().command_eval().shared_snapshot();
        let mru = self.runtime.command_palette_mru();
        let catalog = {
            let commands = self.runtime.commands().lock();
            commands.command_palette_catalog()
        };
        let state = workbench_command_palette_query_state(
            &catalog,
            context.as_ref(),
            "",
            0,
            WindowRequestFocus::First,
            &mru,
        );
        match self.workbench_window_bridge.open_command_palette(state) {
            Ok(true) => {
                self.scene_picker_session = None;
                self.invalidate_host(HostInvalidationMask::PRESENTATION_DATA);
            }
            Ok(false) => {}
            Err(error) => self.set_status_line(error.to_string()),
        }
    }

    pub(in crate::ui::retained_host::app) fn dispatch_workbench_command_palette_query_edited(
        &mut self,
        control_id: &str,
        binding_id: &str,
        query: &str,
    ) -> Option<Result<UiHostEventEffects, String>> {
        if control_id != COMMAND_PALETTE_CONTROL_ID
            || binding_id != COMMAND_PALETTE_QUERY_BINDING_ID
            || self
                .workbench_window_bridge
                .binding_by_id(binding_id)
                .is_none_or(|binding| binding.path().event_kind != EditorUiEventKind::Change)
        {
            return None;
        }

        let context = self.runtime.context().command_eval().shared_snapshot();
        let mru = self.runtime.command_palette_mru();
        let catalog = {
            let commands = self.runtime.commands().lock();
            commands.command_palette_catalog()
        };
        let state = workbench_command_palette_query_state(
            &catalog,
            context.as_ref(),
            query,
            0,
            WindowRequestFocus::First,
            &mru,
        );
        let updated = self
            .workbench_window_bridge
            .update_command_palette_query(state)
            .map_err(|error| error.to_string());
        Some(updated.map(|updated| {
            let mut effects = UiHostEventEffects::default();
            if updated {
                effects.request_paint_only();
            }
            effects
        }))
    }

    pub(in crate::ui::retained_host::app) fn dispatch_workbench_command_palette_window_requested(
        &mut self,
        control_id: &str,
        binding_id: &str,
        value: &str,
    ) -> Option<Result<UiHostEventEffects, String>> {
        if control_id != COMMAND_PALETTE_CONTROL_ID
            || binding_id != COMMAND_PALETTE_WINDOW_BINDING_ID
            || self
                .workbench_window_bridge
                .binding_by_id(binding_id)
                .is_none_or(|binding| binding.path().event_kind != EditorUiEventKind::Change)
        {
            return None;
        }
        let Some((current_offset, target_offset, focus, request_query)) =
            parse_window_request(value)
        else {
            return Some(Err(format!(
                "invalid command palette window request `{value}`"
            )));
        };
        if self.workbench_window_bridge.command_palette_window_offset() != Some(current_offset) {
            return Some(Ok(UiHostEventEffects::default()));
        }
        let query = self.workbench_window_bridge.command_palette_query();
        if query != request_query {
            return Some(Ok(UiHostEventEffects::default()));
        }
        let Some(catalog_generation) = self
            .workbench_window_bridge
            .command_palette_catalog_generation()
        else {
            return Some(Ok(UiHostEventEffects::default()));
        };
        let context = self.runtime.context().command_eval().shared_snapshot();
        let mru = self.runtime.command_palette_mru();
        let catalog = {
            let commands = self.runtime.commands().lock();
            commands.command_palette_catalog()
        };
        if catalog.generation() != catalog_generation {
            return Some(Ok(UiHostEventEffects::default()));
        }
        let window = catalog.query_window_with_mru(
            context.as_ref(),
            &query,
            target_offset,
            COMMAND_PALETTE_VISIBLE_ROWS + COMMAND_PALETTE_OVERSCAN_ROWS,
            &mru,
        );
        let state = workbench_command_palette_state_from_window(query, window, focus);
        let updated = self
            .workbench_window_bridge
            .update_command_palette_query(state)
            .map_err(|error| error.to_string());
        Some(updated.map(|updated| {
            let mut effects = UiHostEventEffects::default();
            if updated {
                effects.request_paint_only();
            }
            effects
        }))
    }
}

#[derive(Clone, Copy)]
enum WindowRequestFocus {
    First,
    Last,
}

fn parse_window_request(value: &str) -> Option<(usize, usize, WindowRequestFocus, String)> {
    let mut fields = value.splitn(4, '|');
    let current_offset = fields.next()?.parse().ok()?;
    let target_offset = fields.next()?.parse().ok()?;
    let focus = fields.next()?;
    let query = fields.next()?.to_string();
    let focus = match focus {
        "first" => WindowRequestFocus::First,
        "last" => WindowRequestFocus::Last,
        _ => return None,
    };
    Some((current_offset, target_offset, focus, query))
}

fn workbench_command_palette_query_state(
    catalog: &std::sync::Arc<EditorCommandPaletteCatalog>,
    context: &CommandEvalCtx,
    query_text: &str,
    offset: usize,
    focus: WindowRequestFocus,
    mru: &EditorCommandPaletteMru,
) -> WorkbenchCommandPaletteOpenState {
    let window = catalog.query_window_with_mru(
        context,
        query_text,
        offset,
        COMMAND_PALETTE_VISIBLE_ROWS + COMMAND_PALETTE_OVERSCAN_ROWS,
        mru,
    );
    workbench_command_palette_state_from_window(query_text.to_owned(), window, focus)
}

fn workbench_command_palette_state_from_window(
    query: String,
    window: EditorCommandPaletteQueryWindow,
    focus: WindowRequestFocus,
) -> WorkbenchCommandPaletteOpenState {
    let focused_index = focused_command_index(&window, focus);
    let selected_command_id = window
        .entries()
        .nth(usize::try_from(focused_index).unwrap_or(usize::MAX))
        .map(|entry| entry.id.clone())
        .unwrap_or_default();

    WorkbenchCommandPaletteOpenState {
        query,
        commands: window.to_ui_value(),
        filtered_commands: UiValue::Array(
            window
                .entries()
                .map(|entry| UiValue::String(entry.id.clone()))
                .collect(),
        ),
        selected_command_id,
        focused_index,
        catalog_generation: window.catalog_generation(),
        total_match_count: window.total_match_count(),
        window_offset: window.offset(),
    }
}

fn focused_command_index(
    query: &EditorCommandPaletteQueryWindow,
    focus: WindowRequestFocus,
) -> i64 {
    let entry = match focus {
        WindowRequestFocus::First => query
            .entries()
            .enumerate()
            .find(|(_, entry)| entry.id != COMMAND_PALETTE_COMMAND_ID)
            .or_else(|| query.entries().enumerate().next()),
        WindowRequestFocus::Last => query
            .entries()
            .enumerate()
            .rev()
            .find(|(_, entry)| entry.id != COMMAND_PALETTE_COMMAND_ID)
            .or_else(|| query.entries().enumerate().last()),
    };
    entry.map(|(index, _)| index as i64).unwrap_or(-1)
}
