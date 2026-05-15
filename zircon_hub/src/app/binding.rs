use crate::settings::{BuildProfile, HubLanguage, HubSettings};
use crate::state::HubSnapshot;

use super::view_model;
use super::HubWindow;

pub(super) fn apply_snapshot(ui: &HubWindow, snapshot: &HubSnapshot) {
    ui.set_selected_page(view_model::selected_page_id(snapshot.selected_page));
    ui.set_selected_page_title(view_model::selected_page_title(snapshot.selected_page));
    ui.set_selected_page_subtitle(view_model::selected_page_subtitle(snapshot.selected_page));
    ui.set_project_sort_label(view_model::project_sort_label(snapshot.project_sort));
    ui.set_project_view_mode(view_model::project_view_mode_id(snapshot.project_view_mode));
    ui.set_search_query(snapshot.search_query.clone().into());
    ui.set_status_label(snapshot.task_status.label.clone().into());
    ui.set_status_detail(snapshot.task_status.detail.clone().into());
    ui.set_task_running(snapshot.task_status.running);
    let project_cards = view_model::project_cards(snapshot);
    let recent_project_rows = view_model::recent_project_rows(snapshot);
    ui.set_nav_items(view_model::model_from(view_model::navigation_items(
        snapshot.selected_page,
    )));
    ui.set_project_card_count(project_cards.len() as i32);
    ui.set_project_cards(view_model::model_from(project_cards));
    ui.set_recent_project_row_count(recent_project_rows.len() as i32);
    ui.set_recent_project_rows(view_model::model_from(recent_project_rows));
    ui.set_quick_actions(view_model::model_from(view_model::quick_actions()));
    ui.set_source_engine(view_model::source_engine_data(
        &snapshot.engines,
        &snapshot.settings,
    ));
    apply_settings(ui, &snapshot.settings);
}

pub(super) fn read_settings(ui: &HubWindow, mut settings: HubSettings) -> HubSettings {
    settings.python_path = ui.get_python_path().to_string();
    settings.cargo_path = ui.get_cargo_path().to_string();
    settings.rustup_path = ui.get_rustup_path().to_string();
    settings.default_project_dir = ui.get_project_location().to_string().into();
    settings.default_source_dir = ui.get_source_path().to_string().into();
    settings.default_build_output_dir = ui.get_output_path().to_string().into();
    settings.build_profile =
        BuildProfile::from_ui_value(&ui.get_build_profile()).unwrap_or(settings.build_profile);
    settings.language = HubLanguage::from_ui_value(&ui.get_language()).unwrap_or(settings.language);
    settings.jobs = ui
        .get_build_jobs()
        .to_string()
        .parse::<u16>()
        .unwrap_or(settings.jobs)
        .max(1);
    settings
}

fn apply_settings(ui: &HubWindow, settings: &HubSettings) {
    ui.set_python_path(settings.python_path.clone().into());
    ui.set_cargo_path(settings.cargo_path.clone().into());
    ui.set_rustup_path(settings.rustup_path.clone().into());
    ui.set_project_location(
        settings
            .default_project_dir
            .to_string_lossy()
            .into_owned()
            .into(),
    );
    ui.set_source_path(
        settings
            .default_source_dir
            .to_string_lossy()
            .into_owned()
            .into(),
    );
    ui.set_output_path(
        settings
            .default_build_output_dir
            .to_string_lossy()
            .into_owned()
            .into(),
    );
    ui.set_build_jobs(settings.jobs.to_string().into());
    ui.set_build_profile(settings.build_profile.as_mode().into());
    ui.set_language(settings.language.as_ui_value().into());
}
