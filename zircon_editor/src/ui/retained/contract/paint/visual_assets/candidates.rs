#[path = "candidates/aliases.rs"]
mod aliases;
#[path = "candidates/paths.rs"]
mod paths;
#[path = "candidates/query.rs"]
mod query;
#[path = "../candidates/resolution.rs"]
mod resolution;
#[path = "../candidates/variants.rs"]
mod variants;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use query::{
    icon_candidates, image_candidates, preview_artifact_candidates,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use resolution::{
    first_existing_path, is_svg_path,
};
