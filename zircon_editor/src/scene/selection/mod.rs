#[path = "domain_selection.rs"]
mod domain_selection;
#[path = "selection_model.rs"]
mod selection_model;
#[path = "selection_mutation.rs"]
mod selection_mutation;

pub use crate::core::play::WorldDomain;
pub use selection_model::SelectionModel;
pub use selection_mutation::SelectionMutation;

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
