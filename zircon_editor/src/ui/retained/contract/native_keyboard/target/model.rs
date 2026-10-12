use super::super::commands::WorkbenchPopupKeyboardCommand;
use super::search::{normalized_popup_text_query, popup_text_starts_with};
use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::primitives::SharedString;

pub(in crate::ui::retained_host::host_contract) struct PopupKeyboardTarget {
    pub(in crate::ui::retained_host::host_contract) control_id: SharedString,
    pub(in crate::ui::retained_host::host_contract) dispatch_kind: SharedString,
    pub(in crate::ui::retained_host::host_contract) rows: Vec<PopupKeyboardRow>,
    pub(in crate::ui::retained_host::host_contract) current_index: usize,
    pub(in crate::ui::retained_host::host_contract) current_row: Option<PopupKeyboardRow>,
    pub(in crate::ui::retained_host::host_contract) current_frame: FrameRect,
    pub(in crate::ui::retained_host::host_contract) popup_frame: FrameRect,
    pub(in crate::ui::retained_host::host_contract) window_offset: usize,
    pub(in crate::ui::retained_host::host_contract) window_count: usize,
    pub(in crate::ui::retained_host::host_contract) total_count: usize,
    pub(in crate::ui::retained_host::host_contract) window_navigation_enabled: bool,
    pub(in crate::ui::retained_host::host_contract) window_query: SharedString,
}

impl PopupKeyboardTarget {
    pub(in crate::ui::retained_host::host_contract) fn next_move(
        &self,
        command: WorkbenchPopupKeyboardCommand,
    ) -> Option<PopupKeyboardMove> {
        if self.rows.is_empty() {
            return None;
        }
        if self.window_navigation_enabled {
            if let Some(request) = self.window_request(command) {
                return Some(PopupKeyboardMove::Window(request));
            }
        }
        self.next_row(command).map(PopupKeyboardMove::Row)
    }

    fn next_row(&self, command: WorkbenchPopupKeyboardCommand) -> Option<PopupKeyboardRow> {
        if self.current_row.is_none() {
            let initial_index = match command {
                WorkbenchPopupKeyboardCommand::Previous
                | WorkbenchPopupKeyboardCommand::Last
                | WorkbenchPopupKeyboardCommand::PageUp => self.rows.len() - 1,
                WorkbenchPopupKeyboardCommand::Next | WorkbenchPopupKeyboardCommand::First => 0,
                WorkbenchPopupKeyboardCommand::PageDown => 0,
                WorkbenchPopupKeyboardCommand::Accept | WorkbenchPopupKeyboardCommand::Cancel => {
                    return None;
                }
            };
            return self.rows.get(initial_index).cloned();
        }
        if self.window_navigation_enabled {
            let next_index = match command {
                WorkbenchPopupKeyboardCommand::Next => self.current_index.checked_add(1),
                WorkbenchPopupKeyboardCommand::Previous => self.current_index.checked_sub(1),
                WorkbenchPopupKeyboardCommand::First => Some(0),
                WorkbenchPopupKeyboardCommand::Last => Some(self.rows.len() - 1),
                WorkbenchPopupKeyboardCommand::PageDown
                | WorkbenchPopupKeyboardCommand::PageUp
                | WorkbenchPopupKeyboardCommand::Accept
                | WorkbenchPopupKeyboardCommand::Cancel => None,
            }?;
            return self.rows.get(next_index).cloned();
        }
        let next_index = match command {
            WorkbenchPopupKeyboardCommand::Next => (self.current_index + 1) % self.rows.len(),
            WorkbenchPopupKeyboardCommand::Previous => {
                (self.current_index + self.rows.len() - 1) % self.rows.len()
            }
            WorkbenchPopupKeyboardCommand::First => 0,
            WorkbenchPopupKeyboardCommand::Last => self.rows.len() - 1,
            WorkbenchPopupKeyboardCommand::PageDown | WorkbenchPopupKeyboardCommand::PageUp => {
                return None;
            }
            WorkbenchPopupKeyboardCommand::Accept | WorkbenchPopupKeyboardCommand::Cancel => {
                self.current_index
            }
        };
        self.rows.get(next_index).cloned()
    }

    fn window_request(
        &self,
        command: WorkbenchPopupKeyboardCommand,
    ) -> Option<PopupKeyboardWindowRequest> {
        match command {
            WorkbenchPopupKeyboardCommand::Next if self.current_index + 1 != self.rows.len() => {
                return None;
            }
            WorkbenchPopupKeyboardCommand::Previous if self.current_index != 0 => return None,
            WorkbenchPopupKeyboardCommand::First | WorkbenchPopupKeyboardCommand::PageUp
                if self.window_offset == 0 =>
            {
                return None;
            }
            WorkbenchPopupKeyboardCommand::Accept | WorkbenchPopupKeyboardCommand::Cancel => {
                return None;
            }
            _ => {}
        }
        let count = self.window_count.max(1);
        let last_offset = self
            .total_count
            .saturating_sub(1)
            .checked_div(count)
            .unwrap_or(0)
            .saturating_mul(count);
        let request = |target_offset, focus| PopupKeyboardWindowRequest {
            current_offset: self.window_offset,
            target_offset,
            focus,
            query: self.window_query.clone(),
        };
        match command {
            WorkbenchPopupKeyboardCommand::Next
                if self.current_index + 1 == self.rows.len()
                    && self.window_offset < last_offset =>
            {
                Some(request(
                    self.window_offset.saturating_add(count).min(last_offset),
                    PopupKeyboardWindowFocus::First,
                ))
            }
            WorkbenchPopupKeyboardCommand::Previous
                if self.current_index == 0 && self.window_offset > 0 =>
            {
                Some(request(
                    self.window_offset.saturating_sub(count),
                    PopupKeyboardWindowFocus::Last,
                ))
            }
            WorkbenchPopupKeyboardCommand::First if self.window_offset > 0 => {
                Some(request(0, PopupKeyboardWindowFocus::First))
            }
            WorkbenchPopupKeyboardCommand::Last if self.window_offset < last_offset => {
                Some(request(last_offset, PopupKeyboardWindowFocus::Last))
            }
            WorkbenchPopupKeyboardCommand::PageDown if self.window_offset < last_offset => {
                Some(request(
                    self.window_offset.saturating_add(count).min(last_offset),
                    PopupKeyboardWindowFocus::First,
                ))
            }
            WorkbenchPopupKeyboardCommand::PageUp if self.window_offset > 0 => Some(request(
                self.window_offset.saturating_sub(count),
                PopupKeyboardWindowFocus::Last,
            )),
            _ => None,
        }
    }

    pub(in crate::ui::retained_host::host_contract) fn text_search_row(
        &self,
        text: &str,
    ) -> Option<PopupKeyboardRow> {
        if self.rows.is_empty() {
            return None;
        }
        let query = normalized_popup_text_query(text)?;
        let start_index = if self.current_row.is_some() {
            (self.current_index + 1) % self.rows.len()
        } else {
            0
        };
        self.rows
            .iter()
            .cycle()
            .skip(start_index)
            .take(self.rows.len())
            .find(|row| row.matches_text_query(&query))
            .cloned()
    }
}

pub(in crate::ui::retained_host::host_contract) enum PopupKeyboardMove {
    Row(PopupKeyboardRow),
    Window(PopupKeyboardWindowRequest),
}

pub(in crate::ui::retained_host::host_contract) struct PopupKeyboardWindowRequest {
    pub(in crate::ui::retained_host::host_contract) current_offset: usize,
    pub(in crate::ui::retained_host::host_contract) target_offset: usize,
    pub(in crate::ui::retained_host::host_contract) focus: PopupKeyboardWindowFocus,
    pub(in crate::ui::retained_host::host_contract) query: SharedString,
}

#[derive(Clone, Copy)]
pub(in crate::ui::retained_host::host_contract) enum PopupKeyboardWindowFocus {
    First,
    Last,
}

#[derive(Clone)]
pub(in crate::ui::retained_host::host_contract) struct PopupKeyboardRow {
    pub(in crate::ui::retained_host::host_contract) action_id: SharedString,
    pub(in crate::ui::retained_host::host_contract) value_text: SharedString,
    pub(in crate::ui::retained_host::host_contract) identity: SharedString,
    pub(in crate::ui::retained_host::host_contract) search_text: SharedString,
    pub(in crate::ui::retained_host::host_contract) focused: bool,
    pub(in crate::ui::retained_host::host_contract) selected: bool,
    pub(in crate::ui::retained_host::host_contract) source_index: Option<usize>,
    pub(in crate::ui::retained_host::host_contract) frame: FrameRect,
}

impl PopupKeyboardRow {
    fn matches_text_query(&self, query: &str) -> bool {
        popup_text_starts_with(&self.search_text, query)
            || popup_text_starts_with(&self.value_text, query)
            || popup_text_starts_with(&self.identity, query)
    }
}

#[cfg(test)]
#[path = "tests/model.rs"]
mod tests;
