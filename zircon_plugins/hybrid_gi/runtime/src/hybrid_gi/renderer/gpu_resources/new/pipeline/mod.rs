pub(in crate::hybrid_gi::renderer::gpu_resources::new) mod pipeline;

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;

pub(super) use pipeline::pipeline;
