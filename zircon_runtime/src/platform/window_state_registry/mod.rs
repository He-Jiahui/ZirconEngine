mod registry;
mod window_state_registry_error;

pub(crate) use registry::WindowStateRegistry;
pub(crate) use window_state_registry_error::WindowStateRegistryError;

#[cfg(test)]
mod tests;
