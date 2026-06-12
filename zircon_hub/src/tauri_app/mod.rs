mod action_id;
mod action_request;
mod commands;
mod runtime_state;
mod view_model;

pub(crate) use action_request::HubActionRequest;
use commands::HubCommandState;
pub(crate) use view_model::HubViewModel;

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
    tauri::Builder::default()
        .manage(HubCommandState::load()?)
        .invoke_handler(tauri::generate_handler![hub_state, hub_action])
        .run(tauri::generate_context!())?;
    Ok(())
}
