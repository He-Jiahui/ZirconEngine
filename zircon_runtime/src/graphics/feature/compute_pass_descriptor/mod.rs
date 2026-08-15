mod compute_pass_descriptor;
mod compute_shader_source;
mod lowering;

pub use compute_pass_descriptor::ComputePassDescriptor;
pub use compute_shader_source::ComputeShaderSource;
pub use lowering::COMPUTE_GENERIC_EXECUTOR_ID;

#[cfg(test)]
mod tests;
