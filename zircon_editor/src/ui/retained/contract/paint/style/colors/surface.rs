#[path = "surface/entry.rs"]
mod entry;
#[path = "surface/interaction.rs"]
mod interaction;
#[path = "surface/severity.rs"]
mod severity;
#[path = "surface/variants.rs"]
mod variants;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use entry::surface_color;
