#[path = "../../compute_pass_descriptor/compute_pass_descriptor.rs"]
mod compute_pass_descriptor;
#[path = "../../compute_pass_descriptor/compute_shader_source.rs"]
mod compute_shader_source;
#[path = "lowering.rs"]
mod lowering;

pub use compute_pass_descriptor::ComputePassDescriptor;
pub use compute_shader_source::ComputeShaderSource;
pub use lowering::COMPUTE_GENERIC_EXECUTOR_ID;

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
