pub const EXPOSURE_READBACK_EXPECTED_BYTE_LEN: usize = 16;

/// 曝光缓冲读回的帧后诊断；按 GPU 的四个 f32 字布局保存位模式以供历史有效性核对。
/// 读回可缺席或长度异常，消费者应先检查 `history_valid` 再比较曝光乘数。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderExposureReadbackReport {
    pub available: bool,
    pub byte_len: usize,
    pub expected_byte_len: usize,
    pub invalid_byte_len: bool,
    pub invalid_word_count: usize,
    pub multiplier_bits: u32,
    pub resolved_ev100_bits: u32,
    pub average_ev100_bits: u32,
    pub valid_flag_bits: u32,
}

impl RenderExposureReadbackReport {
    pub fn from_words(words: [f32; 4]) -> Self {
        Self::from_raw_f32x4_bytes(&f32_words_to_le_bytes(words))
    }

    pub fn from_raw_f32x4_bytes(bytes: &[u8]) -> Self {
        let mut words = [0.0_f32; 4];
        let mut invalid_word_count = 0;
        for (word, chunk) in words.iter_mut().zip(bytes.chunks_exact(4)) {
            let decoded = f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
            invalid_word_count += (!decoded.is_finite()) as usize;
            *word = decoded;
        }
        Self {
            available: true,
            byte_len: bytes.len(),
            expected_byte_len: EXPOSURE_READBACK_EXPECTED_BYTE_LEN,
            invalid_byte_len: bytes.len() != EXPOSURE_READBACK_EXPECTED_BYTE_LEN,
            invalid_word_count,
            multiplier_bits: words[0].to_bits(),
            resolved_ev100_bits: words[1].to_bits(),
            average_ev100_bits: words[2].to_bits(),
            valid_flag_bits: words[3].to_bits(),
        }
    }

    pub fn multiplier(self) -> f32 {
        f32::from_bits(self.multiplier_bits)
    }

    pub fn resolved_ev100(self) -> f32 {
        f32::from_bits(self.resolved_ev100_bits)
    }

    pub fn average_ev100(self) -> f32 {
        f32::from_bits(self.average_ev100_bits)
    }

    pub fn valid_flag(self) -> f32 {
        f32::from_bits(self.valid_flag_bits)
    }

    pub fn multiplier_micro(self) -> usize {
        f32_to_nonnegative_micro(self.multiplier())
    }

    pub fn valid_flag_micro(self) -> usize {
        f32_to_nonnegative_micro(self.valid_flag())
    }

    pub fn history_valid(self) -> bool {
        self.available
            && !self.invalid_byte_len
            && self.invalid_word_count == 0
            && self.multiplier().is_finite()
            && self.multiplier() >= 0.0
            && self.valid_flag() > 0.5
    }

    pub fn multiplier_within_epsilon(self, expected: f32, epsilon: f32) -> bool {
        self.history_valid() && (self.multiplier() - expected).abs() <= epsilon
    }
}

fn f32_words_to_le_bytes(words: [f32; 4]) -> [u8; EXPOSURE_READBACK_EXPECTED_BYTE_LEN] {
    let mut bytes = [0_u8; EXPOSURE_READBACK_EXPECTED_BYTE_LEN];
    for (index, word) in words.into_iter().enumerate() {
        bytes[index * 4..index * 4 + 4].copy_from_slice(&word.to_le_bytes());
    }
    bytes
}

fn f32_to_nonnegative_micro(value: f32) -> usize {
    if !value.is_finite() || value <= 0.0 {
        return 0;
    }
    (value * 1_000_000.0).round() as usize
}

#[cfg(test)]
#[path = "tests/exposure_readback.rs"]
mod tests;

#[cfg(test)]
#[path = "exposure_readback/tests/single_pass_tests.rs"]
mod single_pass_tests;
