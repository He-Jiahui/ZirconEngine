#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() -> Result<(), zircon_hub::HubError> {
    zircon_hub::tauri_app::run()
}
