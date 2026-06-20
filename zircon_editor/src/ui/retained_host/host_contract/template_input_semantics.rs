mod classification;
mod target;

pub(in crate::ui::retained_host::host_contract) use classification::hit_is_text_input;
pub(in crate::ui::retained_host::host_contract) use target::text_input_edit_target_id;

#[cfg(test)]
#[path = "template_input_semantics_tests.rs"]
mod tests;
