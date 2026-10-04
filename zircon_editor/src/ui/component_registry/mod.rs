mod registry;

pub(crate) use registry::retained_component_registry;

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
