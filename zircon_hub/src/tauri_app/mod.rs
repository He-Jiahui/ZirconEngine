mod account_commands;
mod action_id;
mod action_request;
mod commands;
mod runtime_state;
mod view_model;
mod window_state;

use tauri::Manager;

use account_commands::AccountCommandState;
pub(crate) use action_request::HubActionRequest;
use commands::HubCommandState;
pub(crate) use view_model::HubViewModel;

#[tauri::command]
async fn account_state(
    hub: tauri::State<'_, HubCommandState>,
    account: tauri::State<'_, AccountCommandState>,
) -> Result<account_commands::AccountSnapshot, String> {
    account_commands::account_state(hub, account)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn account_action(
    request: account_commands::AccountActionRequest,
    hub: tauri::State<'_, HubCommandState>,
    account: tauri::State<'_, AccountCommandState>,
) -> Result<account_commands::AccountSnapshot, String> {
    tauri::async_runtime::block_on(account_commands::account_action(request, hub, account))
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn hub_state(state: tauri::State<'_, HubCommandState>) -> Result<HubViewModel, String> {
    commands::hub_state(state).map_err(|error| error.to_string())
}

#[tauri::command]
fn hub_action(
    request: HubActionRequest,
    state: tauri::State<'_, HubCommandState>,
    app: tauri::AppHandle,
) -> Result<HubViewModel, String> {
    commands::hub_action(request, state, app).map_err(|error| error.to_string())
}

pub fn run() -> Result<(), crate::HubError> {
    let hub_commands = HubCommandState::load()?;
    let editor_shutdown = hub_commands.editor_shutdown_handle();
    let run_result = tauri::Builder::default()
        .manage(hub_commands)
        .manage(AccountCommandState::load())
        .manage(window_state::WindowGeometryCapture::default())
        .setup(|app| {
            app.state::<HubCommandState>()
                .install_editor_terminal_emitter(app.handle().clone());
            window_state::restore_saved_window_state(app)
        })
        .on_window_event(|window, event| {
            window_state::handle_window_event(window, event);
            if matches!(event, tauri::WindowEvent::Focused(true)) {
                window
                    .state::<HubCommandState>()
                    .refresh_recent_projects_on_window_focus(window.app_handle().clone());
            }
        })
        .invoke_handler(tauri::generate_handler![
            hub_state,
            hub_action,
            account_state,
            account_action
        ])
        .run(tauri::generate_context!());
    let shutdown_result = editor_shutdown.shutdown();
    run_result?;
    shutdown_result?;
    Ok(())
}
