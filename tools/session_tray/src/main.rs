#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() -> Result<(), zircon_session_tray::TrayError> {
    zircon_session_tray::run()
}
