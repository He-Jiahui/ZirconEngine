mod blocking;
mod dimensions;

pub(in crate::ui::retained_host::host_contract) use self::blocking::popup_blocking_frame;
pub(in crate::ui::retained_host::host_contract) use self::dimensions::{
    shell_content_height, shell_content_width,
};
