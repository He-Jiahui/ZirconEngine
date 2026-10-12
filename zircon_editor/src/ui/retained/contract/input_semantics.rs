#[path = "input_semantics/classification.rs"]
mod classification;
#[path = "input_semantics/target.rs"]
mod target;

pub(in crate::ui::retained_host::host_contract) use classification::hit_is_text_input;
pub(in crate::ui::retained_host::host_contract) use target::text_input_edit_target_id;

#[cfg(test)]
#[path = "tests/input_semantics.rs"]
mod tests;
