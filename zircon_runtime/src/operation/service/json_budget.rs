use std::io::{self, Write};

use super::super::RuntimeOperationServiceError;

struct JsonByteCounter(usize);

impl Write for JsonByteCounter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0 = self.0.saturating_add(bytes.len());
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub(super) fn json_value_byte_len(
    value: &serde_json::Value,
) -> Result<usize, RuntimeOperationServiceError> {
    let mut counter = JsonByteCounter(0);
    serde_json::to_writer(&mut counter, value).map_err(|error| {
        RuntimeOperationServiceError::PayloadEncoding {
            message: error.to_string(),
        }
    })?;
    Ok(counter.0)
}

pub(super) fn truncate_utf8_to_bytes(message: String, maximum: usize) -> String {
    if message.len() <= maximum {
        return message;
    }
    let mut end = maximum;
    while end > 0 && !message.is_char_boundary(end) {
        end -= 1;
    }
    message[..end].to_owned()
}
