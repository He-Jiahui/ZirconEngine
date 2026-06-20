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
}

impl PopupKeyboardTarget {
    pub(in crate::ui::retained_host::host_contract) fn next_row(
        &self,
        command: WorkbenchPopupKeyboardCommand,
    ) -> Option<PopupKeyboardRow> {
        if self.rows.is_empty() {
            return None;
        }
        let next_index = match command {
            WorkbenchPopupKeyboardCommand::Next => (self.current_index + 1) % self.rows.len(),
            WorkbenchPopupKeyboardCommand::Previous => {
                (self.current_index + self.rows.len() - 1) % self.rows.len()
            }
            WorkbenchPopupKeyboardCommand::First => 0,
            WorkbenchPopupKeyboardCommand::Last => self.rows.len() - 1,
            WorkbenchPopupKeyboardCommand::Accept | WorkbenchPopupKeyboardCommand::Cancel => {
                self.current_index
            }
        };
        self.rows.get(next_index).cloned()
    }

    pub(in crate::ui::retained_host::host_contract) fn text_search_row(
        &self,
        text: &str,
    ) -> Option<PopupKeyboardRow> {
        if self.rows.is_empty() {
            return None;
        }
        let query = normalized_popup_text_query(text)?;
        let start_index = (self.current_index + 1) % self.rows.len();
        self.rows
            .iter()
            .cycle()
            .skip(start_index)
            .take(self.rows.len())
            .find(|row| row.matches_text_query(&query))
            .cloned()
    }
}

#[derive(Clone)]
pub(in crate::ui::retained_host::host_contract) struct PopupKeyboardRow {
    pub(in crate::ui::retained_host::host_contract) action_id: SharedString,
    pub(in crate::ui::retained_host::host_contract) value_text: SharedString,
    pub(in crate::ui::retained_host::host_contract) identity: SharedString,
    pub(in crate::ui::retained_host::host_contract) search_text: SharedString,
    pub(in crate::ui::retained_host::host_contract) focused: bool,
    pub(in crate::ui::retained_host::host_contract) selected: bool,
    pub(in crate::ui::retained_host::host_contract) frame: FrameRect,
}

impl PopupKeyboardRow {
    fn matches_text_query(&self, query: &str) -> bool {
        popup_text_starts_with(&self.search_text, query)
            || popup_text_starts_with(&self.value_text, query)
            || popup_text_starts_with(&self.identity, query)
    }
}
