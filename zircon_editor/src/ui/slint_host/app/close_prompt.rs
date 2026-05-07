use crate::ui::slint_host::{FrameRect, HostClosePromptData, UiHostWindow};
use crate::ui::workbench::layout::MainPageId;
use crate::ui::workbench::view::{ViewDescriptorId, ViewInstance, ViewInstanceId};

const BUTTON_WIDTH: f32 = 88.0;
const BUTTON_HEIGHT: f32 = 30.0;
const BUTTON_GAP: f32 = 10.0;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum ClosePromptTarget {
    MainWindow,
    FloatingWindow(MainPageId),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct DirtyCloseView {
    pub instance_id: ViewInstanceId,
    pub descriptor_id: ViewDescriptorId,
    pub title: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct PendingClosePrompt {
    pub target: ClosePromptTarget,
    pub close_instances: Vec<ViewInstanceId>,
    pub dirty_views: Vec<DirtyCloseView>,
}

impl PendingClosePrompt {
    pub fn new(
        target: ClosePromptTarget,
        close_instances: Vec<ViewInstanceId>,
        dirty_views: Vec<DirtyCloseView>,
    ) -> Self {
        Self {
            target,
            close_instances,
            dirty_views,
        }
    }
}

pub(super) fn dirty_close_views(
    instances: &[ViewInstance],
    candidate_ids: impl IntoIterator<Item = ViewInstanceId>,
) -> Vec<DirtyCloseView> {
    let candidates = candidate_ids.into_iter().collect::<Vec<_>>();
    instances
        .iter()
        .filter(|instance| {
            instance.dirty && candidates.iter().any(|id| id == &instance.instance_id)
        })
        .map(|instance| DirtyCloseView {
            instance_id: instance.instance_id.clone(),
            descriptor_id: instance.descriptor_id.clone(),
            title: instance.title.clone(),
        })
        .collect()
}

pub(super) fn all_dirty_close_views(instances: &[ViewInstance]) -> Vec<DirtyCloseView> {
    instances
        .iter()
        .filter(|instance| instance.dirty)
        .map(|instance| DirtyCloseView {
            instance_id: instance.instance_id.clone(),
            descriptor_id: instance.descriptor_id.clone(),
            title: instance.title.clone(),
        })
        .collect()
}

pub(super) fn show_prompt(ui: &UiHostWindow, prompt: &PendingClosePrompt) {
    ui.set_close_prompt(host_prompt_data(ui, prompt));
}

pub(super) fn clear_prompt(ui: &UiHostWindow) {
    ui.clear_close_prompt();
}

fn host_prompt_data(ui: &UiHostWindow, prompt: &PendingClosePrompt) -> HostClosePromptData {
    let size = ui.window().size();
    let width = size.width as f32;
    let height = size.height as f32;
    let dialog_width = (width - 48.0).clamp(280.0, 500.0);
    let dialog_height = 176.0;
    let dialog = FrameRect {
        x: ((width - dialog_width) * 0.5).max(16.0),
        y: ((height - dialog_height) * 0.5).max(16.0),
        width: dialog_width,
        height: dialog_height,
    };
    let cancel = button_frame(&dialog, 0);
    let discard = button_frame(&dialog, 1);
    let save = button_frame(&dialog, 2);
    HostClosePromptData {
        visible: true,
        target_window_id: target_window_id(&prompt.target).into(),
        title: prompt_title(&prompt.target).into(),
        message: prompt_message(prompt.dirty_views.len()).into(),
        details: dirty_details(&prompt.dirty_views).into(),
        can_save: prompt.dirty_views.iter().all(can_save_dirty_view),
        overlay_frame: FrameRect {
            x: 0.0,
            y: 0.0,
            width,
            height,
        },
        dialog_frame: dialog,
        save_button_frame: save,
        discard_button_frame: discard,
        cancel_button_frame: cancel,
    }
}

fn button_frame(dialog: &FrameRect, reverse_index: usize) -> FrameRect {
    let right = dialog.x + dialog.width - 18.0;
    FrameRect {
        x: right - BUTTON_WIDTH - reverse_index as f32 * (BUTTON_WIDTH + BUTTON_GAP),
        y: dialog.y + dialog.height - BUTTON_HEIGHT - 18.0,
        width: BUTTON_WIDTH,
        height: BUTTON_HEIGHT,
    }
}

fn target_window_id(target: &ClosePromptTarget) -> String {
    match target {
        ClosePromptTarget::MainWindow => "main".to_string(),
        ClosePromptTarget::FloatingWindow(window_id) => window_id.0.clone(),
    }
}

fn prompt_title(target: &ClosePromptTarget) -> &'static str {
    match target {
        ClosePromptTarget::MainWindow => "Save changes before closing Zircon?",
        ClosePromptTarget::FloatingWindow(_) => "Save changes before closing window?",
    }
}

fn prompt_message(count: usize) -> String {
    match count {
        0 => "No dirty documents are pending.".to_string(),
        1 => "One modified document has unsaved changes.".to_string(),
        count => format!("{count} modified documents have unsaved changes."),
    }
}

fn dirty_details(views: &[DirtyCloseView]) -> String {
    let mut names = views
        .iter()
        .take(3)
        .map(|view| view.title.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    if views.len() > 3 {
        names.push_str(", ...");
    }
    names
}

pub(super) fn close_action_id(action: &str) -> Option<&'static str> {
    match action {
        "save" => Some("save"),
        "discard" => Some("discard"),
        "cancel" => Some("cancel"),
        _ => None,
    }
}

pub(super) fn can_save_dirty_view(view: &DirtyCloseView) -> bool {
    matches!(
        view.descriptor_id.0.as_str(),
        "editor.ui_asset" | "editor.animation_sequence" | "editor.animation_graph"
    )
}
