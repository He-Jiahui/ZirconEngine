pub(super) struct BoundedLineDecoder {
    bytes: Vec<u8>,
    max_bytes: usize,
    truncated_bytes: u64,
}

pub(super) struct DecodedLine {
    pub text: String,
    pub source_bytes: u64,
    pub truncated_bytes: u64,
    pub lossy_utf8: bool,
}

impl BoundedLineDecoder {
    pub fn new(max_bytes: usize) -> Self {
        Self {
            bytes: Vec::with_capacity(max_bytes),
            max_bytes,
            truncated_bytes: 0,
        }
    }

    pub fn push(&mut self, input: &[u8], mut emit: impl FnMut(DecodedLine) -> bool) -> bool {
        for byte in input {
            if *byte == b'\n' {
                if !emit(self.finish_line()) {
                    return false;
                }
            } else if self.bytes.len() < self.max_bytes {
                self.bytes.push(*byte);
            } else {
                self.truncated_bytes = self.truncated_bytes.saturating_add(1);
            }
        }
        true
    }

    pub fn finish(&mut self) -> Option<DecodedLine> {
        (!self.bytes.is_empty() || self.truncated_bytes > 0).then(|| self.finish_line())
    }

    fn finish_line(&mut self) -> DecodedLine {
        if self.bytes.last() == Some(&b'\r') {
            self.bytes.pop();
        }
        let source_bytes = (self.bytes.len() as u64).saturating_add(self.truncated_bytes);
        let (text, lossy_utf8, rendered_omitted_bytes) =
            decode_lossy_bounded(&self.bytes, self.max_bytes);
        let line = DecodedLine {
            text,
            source_bytes,
            truncated_bytes: self
                .truncated_bytes
                .saturating_add(rendered_omitted_bytes as u64),
            lossy_utf8,
        };
        self.bytes.clear();
        self.truncated_bytes = 0;
        line
    }
}

fn decode_lossy_bounded(input: &[u8], byte_limit: usize) -> (String, bool, usize) {
    let mut output = String::with_capacity(input.len().min(byte_limit));
    let mut cursor = 0;
    let mut lossy = false;

    while cursor < input.len() {
        match std::str::from_utf8(&input[cursor..]) {
            Ok(valid) => {
                let accepted = bounded_valid_prefix(valid, byte_limit.saturating_sub(output.len()));
                output.push_str(&valid[..accepted]);
                cursor += accepted;
                break;
            }
            Err(error) => {
                lossy = true;
                let valid_bytes = error.valid_up_to();
                let valid = match std::str::from_utf8(&input[cursor..cursor + valid_bytes]) {
                    Ok(valid) => valid,
                    Err(_) => {
                        return (output, true, input.len().saturating_sub(cursor));
                    }
                };
                let accepted = bounded_valid_prefix(valid, byte_limit.saturating_sub(output.len()));
                output.push_str(&valid[..accepted]);
                cursor += accepted;
                if accepted < valid_bytes {
                    break;
                }

                match error.error_len() {
                    Some(invalid_bytes) => {
                        if output
                            .len()
                            .saturating_add(char::REPLACEMENT_CHARACTER.len_utf8())
                            > byte_limit
                        {
                            break;
                        }
                        output.push(char::REPLACEMENT_CHARACTER);
                        cursor += invalid_bytes;
                    }
                    None => {
                        if output
                            .len()
                            .saturating_add(char::REPLACEMENT_CHARACTER.len_utf8())
                            <= byte_limit
                        {
                            output.push(char::REPLACEMENT_CHARACTER);
                            cursor = input.len();
                        }
                        break;
                    }
                }
            }
        }
    }

    (output, lossy, input.len().saturating_sub(cursor))
}

fn bounded_valid_prefix(value: &str, byte_limit: usize) -> usize {
    if value.len() <= byte_limit {
        return value.len();
    }
    let mut end = byte_limit;
    while !value.is_char_boundary(end) {
        end -= 1;
    }
    end
}
