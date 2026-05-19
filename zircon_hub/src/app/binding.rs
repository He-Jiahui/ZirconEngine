use crate::settings::{BuildProfile, HubLanguage, HubSettings};
use crate::state::HubSnapshot;

use super::view_model;
use super::HubWindow;

pub(super) fn apply_snapshot(ui: &HubWindow, snapshot: &HubSnapshot) {
    let language = snapshot.settings.language;
    ui.set_ui_text(view_model::ui_text(language));
    ui.set_selected_page(view_model::selected_page_id(snapshot.selected_page));
    ui.set_selected_page_title(view_model::selected_page_title(
        snapshot.selected_page,
        language,
    ));
    ui.set_selected_page_subtitle(view_model::selected_page_subtitle(
        snapshot.selected_page,
        language,
    ));
    ui.set_project_filter_label(view_model::project_filter_label(
        snapshot.project_filter,
        language,
    ));
    ui.set_project_sort_label(view_model::project_sort_label(
        snapshot.project_sort,
        language,
    ));
    ui.set_project_subpage(view_model::project_subpage_id(snapshot.project_subpage));
    ui.set_project_view_mode(view_model::project_view_mode_id(snapshot.project_view_mode));
    ui.set_search_query(snapshot.search_query.clone().into());
    ui.set_status_label(snapshot.task_status.label.clone().into());
    ui.set_status_detail(snapshot.task_status.detail.clone().into());
    ui.set_task_running(snapshot.task_status.running);
    ui.set_header_statuses(view_model::model_from(view_model::header_statuses(
        snapshot,
    )));
    let project_cards = view_model::project_cards(snapshot);
    let project_list_rows = view_model::project_list_rows(snapshot);
    let project_browser_rows = view_model::project_browser_rows(snapshot);
    let dashboard_project_rows = view_model::dashboard_project_rows(snapshot);
    let project_templates = view_model::project_templates(snapshot);
    let project_engine_rows = view_model::project_engine_rows(snapshot);
    let recent_project_rows = view_model::recent_project_rows(snapshot);
    let asset_items = view_model::asset_items(snapshot);
    let learn_items = view_model::learn_items(snapshot);
    let plugin_items = view_model::plugin_items(snapshot);
    let team_members = view_model::team_members(snapshot);
    let cloud_services = view_model::cloud_services(language);
    ui.set_nav_items(view_model::model_from(view_model::navigation_items(
        snapshot.selected_page,
        language,
    )));
    ui.set_project_card_count(project_cards.len() as i32);
    ui.set_project_cards(view_model::model_from(project_cards));
    ui.set_project_list_row_count(project_list_rows.len() as i32);
    ui.set_project_list_rows(view_model::model_from(project_list_rows));
    ui.set_project_browser_row_count(project_browser_rows.len() as i32);
    ui.set_project_browser_rows(view_model::model_from(project_browser_rows));
    ui.set_dashboard_project_title(view_model::dashboard_project_title(snapshot, language));
    ui.set_dashboard_project_row_count(dashboard_project_rows.len() as i32);
    ui.set_dashboard_project_rows(view_model::model_from(dashboard_project_rows));
    ui.set_project_template_count(project_templates.len() as i32);
    ui.set_project_templates(view_model::model_from(project_templates));
    ui.set_project_create_enabled(view_model::project_create_enabled(snapshot));
    ui.set_project_create_template_label(view_model::project_create_template_label(snapshot));
    ui.set_project_create_engine_label(view_model::project_create_engine_label(snapshot));
    ui.set_project_engine_count(project_engine_rows.len() as i32);
    ui.set_project_engine_rows(view_model::model_from(project_engine_rows));
    ui.set_project_detail(view_model::project_detail(snapshot));
    ui.set_recent_project_row_count(recent_project_rows.len() as i32);
    ui.set_recent_project_rows(view_model::model_from(recent_project_rows));
    ui.set_quick_actions(view_model::model_from(view_model::quick_actions(language)));
    ui.set_asset_count(asset_items.len() as i32);
    ui.set_assets(view_model::model_from(asset_items));
    ui.set_learn_count(learn_items.len() as i32);
    ui.set_learn_resources(view_model::model_from(learn_items));
    ui.set_plugin_count(plugin_items.len() as i32);
    ui.set_plugins(view_model::model_from(plugin_items));
    ui.set_team_summary(view_model::team_summary(snapshot));
    ui.set_team_member_count(team_members.len() as i32);
    ui.set_team_members(view_model::model_from(team_members));
    ui.set_cloud_summary(view_model::cloud_summary(snapshot));
    ui.set_cloud_service_count(cloud_services.len() as i32);
    ui.set_cloud_services(view_model::model_from(cloud_services));
    let source_engine_rows = view_model::source_engine_rows(snapshot);
    let source_build_history_rows = view_model::source_build_history_rows(snapshot);
    ui.set_source_engine_count(source_engine_rows.len() as i32);
    ui.set_source_engines(view_model::model_from(source_engine_rows));
    ui.set_source_build_history_count(source_build_history_rows.len() as i32);
    ui.set_source_build_history(view_model::model_from(source_build_history_rows));
    ui.set_settings_statuses(view_model::model_from(view_model::settings_statuses(
        &snapshot.settings,
    )));
    let source_engine = view_model::source_engine_data(
        &snapshot.engines,
        &snapshot.settings,
        snapshot.active_engine_id.as_deref(),
    );
    ui.set_active_engine_name(source_engine.title.clone());
    ui.set_source_engine(source_engine);
    apply_settings(ui, &snapshot.settings);
    reset_page_scroll_offsets(ui);
}

pub(super) fn read_settings(ui: &HubWindow, mut settings: HubSettings) -> HubSettings {
    settings.python_path = ui.get_python_path().to_string();
    settings.cargo_path = ui.get_cargo_path().to_string();
    settings.rustup_path = ui.get_rustup_path().to_string();
    settings.default_project_dir = ui.get_project_location().to_string().into();
    settings.default_source_dir = ui.get_source_path().to_string().into();
    settings.default_build_output_dir = ui.get_output_path().to_string().into();
    settings.default_device_install_dir = ui.get_device_install_path().to_string().into();
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
    ui.set_device_install_path(
        settings
            .default_device_install_dir
            .to_string_lossy()
            .into_owned()
            .into(),
    );
    ui.set_build_jobs(settings.jobs.to_string().into());
    ui.set_build_profile(settings.build_profile.as_mode().into());
    ui.set_language(settings.language.as_ui_value().into());
}

fn reset_page_scroll_offsets(ui: &HubWindow) {
    ui.set_editor_scroll_y(0.0);
    ui.set_assets_scroll_y(0.0);
    ui.set_builds_scroll_y(0.0);
    ui.set_plugins_scroll_y(0.0);
    ui.set_cloud_scroll_y(0.0);
    ui.set_team_scroll_y(0.0);
    ui.set_learn_scroll_y(0.0);
    ui.set_settings_scroll_y(0.0);
}
