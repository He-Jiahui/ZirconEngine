use super::*;

#[path = "structure/layout.rs"]
mod layout;
#[path = "structure/semantic_paths.rs"]
mod semantic_paths;
#[path = "structure/slot.rs"]
mod slot;

#[cfg(test)]
#[path = "structure/tests/cases.rs"]
mod tests;
