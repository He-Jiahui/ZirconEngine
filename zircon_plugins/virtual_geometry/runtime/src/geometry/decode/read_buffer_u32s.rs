use zircon_runtime::graphics::GraphicsError;

pub(super) fn read_buffer_u32s(bytes: &[u8], word_count: usize) -> Result<Vec<u32>, GraphicsError> {
    if word_count == 0 {
        return Ok(Vec::new());
    }

    let byte_len = word_count.saturating_mul(std::mem::size_of::<u32>());
    if bytes.len() < byte_len {
        return Err(GraphicsError::BufferMap(format!(
            "virtual geometry readback returned {} bytes for {word_count} words",
            bytes.len()
        )));
    }
    Ok(bytes[..byte_len]
        .chunks_exact(4)
        .map(|word| u32::from_le_bytes([word[0], word[1], word[2], word[3]]))
        .collect())
}
