const GEMM_PARAMETER_BYTES: usize = 32;
const ELEMENTWISE_PARAMETER_BYTES: usize = 16;

pub(super) fn gemm_parameters(m: u32, n: u32, k: u32, alpha: f32, beta: f32) -> Vec<u8> {
    let mut parameters = [0; GEMM_PARAMETER_BYTES];
    parameters[0..4].copy_from_slice(&m.to_le_bytes());
    parameters[4..8].copy_from_slice(&n.to_le_bytes());
    parameters[8..12].copy_from_slice(&k.to_le_bytes());
    parameters[16..20].copy_from_slice(&alpha.to_le_bytes());
    parameters[20..24].copy_from_slice(&beta.to_le_bytes());
    parameters.to_vec()
}

pub(super) fn elementwise_parameters(elements: u32) -> Vec<u8> {
    let mut parameters = [0; ELEMENTWISE_PARAMETER_BYTES];
    parameters[0..4].copy_from_slice(&elements.to_le_bytes());
    parameters.to_vec()
}

#[cfg(test)]
mod performance_tests;
