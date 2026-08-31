use super::super::model::{ClosePromptTarget, DirtyCloseView};

pub(super) fn target_window_id(target: &ClosePromptTarget) -> String {
    match target {
        ClosePromptTarget::Project | ClosePromptTarget::MainWindow => "main".to_string(),
        ClosePromptTarget::FloatingWindow(window_id) => window_id.0.clone(),
    }
}

pub(super) fn prompt_title(target: &ClosePromptTarget) -> &'static str {
    match target {
        ClosePromptTarget::Project => "Save changes before closing project?",
        ClosePromptTarget::MainWindow => "Save changes before closing Zircon?",
        ClosePromptTarget::FloatingWindow(_) => "Save changes before closing window?",
    }
}

pub(super) fn prompt_message(count: usize) -> String {
    match count {
        0 => "No dirty documents are pending.".to_string(),
        1 => "One modified document has unsaved changes.".to_string(),
        count => format!("{count} modified documents have unsaved changes."),
    }
}

pub(super) fn dirty_details(views: &[DirtyCloseView], includes_project_scene: bool) -> String {
    let mut names = includes_project_scene
        .then_some("Active Scene")
        .into_iter()
        .chain(views.iter().map(|view| view.title.as_str()))
        .take(3)
        .collect::<Vec<_>>()
        .join(", ");
    if views.len() + usize::from(includes_project_scene) > 3 {
        names.push_str(", ...");
    }
    names
}
