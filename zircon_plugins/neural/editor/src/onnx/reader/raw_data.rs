use std::mem::size_of;

use super::OnnxReadError;

pub(super) fn decode_f32(bytes: &[u8]) -> Result<Vec<f32>, OnnxReadError> {
    if bytes.len() % size_of::<f32>() != 0 {
        return Err(OnnxReadError::InvalidFloatTensorData);
    }
    Ok(bytes
        .chunks_exact(size_of::<f32>())
        .map(|value| f32::from_le_bytes(value.try_into().unwrap()))
        .collect())
}

#[cfg(test)]
mod performance_tests;
