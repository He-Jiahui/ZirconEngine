use gilrs::GilrsBuilder;
use zircon_runtime::diagnostic_log::write_warn;

pub(in crate::entry::runtime_entry_app) fn create_gilrs() -> Option<gilrs::Gilrs> {
    match GilrsBuilder::new()
        .with_default_filters(false)
        .set_update_state(false)
        .build()
    {
        Ok(gilrs) => Some(gilrs),
        Err(error) => {
            write_warn(
                "runtime_gamepad",
                format!("runtime_gamepad_gilrs_unavailable: {error}"),
            );
            None
        }
    }
}
