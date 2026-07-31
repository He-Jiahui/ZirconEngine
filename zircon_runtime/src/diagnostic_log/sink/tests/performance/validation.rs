const WORD_BITS: usize = u64::BITS as usize;

#[derive(Clone, Copy)]
pub(super) struct ValidationSnapshot {
    pub(super) records: u64,
    pub(super) duplicates: u64,
    pub(super) malformed: u64,
}

pub(super) struct LineValidation {
    expected_records: usize,
    seen: Vec<u64>,
    records: u64,
    duplicates: u64,
    malformed: u64,
}

impl LineValidation {
    pub(super) fn new(expected_records: usize) -> Self {
        Self {
            expected_records,
            seen: vec![0; expected_records.div_ceil(WORD_BITS)],
            records: 0,
            duplicates: 0,
            malformed: 0,
        }
    }

    pub(super) fn observe(&mut self, buffer: &[u8]) {
        for bytes in buffer
            .split(|byte| *byte == b'\n')
            .filter(|line| !line.is_empty())
        {
            let Some(sequence) = parse_sequence(bytes) else {
                self.malformed += 1;
                continue;
            };
            if sequence >= self.expected_records {
                self.malformed += 1;
                continue;
            }
            let word = sequence / WORD_BITS;
            let mask = 1u64 << (sequence % WORD_BITS);
            if self.seen[word] & mask != 0 {
                self.duplicates += 1;
                continue;
            }
            self.seen[word] |= mask;
            self.records += 1;
        }
    }

    pub(super) fn snapshot(&self) -> ValidationSnapshot {
        ValidationSnapshot {
            records: self.records,
            duplicates: self.duplicates,
            malformed: self.malformed,
        }
    }
}

fn parse_sequence(bytes: &[u8]) -> Option<usize> {
    let line = std::str::from_utf8(bytes).ok()?;
    let value = line.split_once("record=")?.1;
    let digits = value.split_once(' ').map_or(value, |(digits, _)| digits);
    (!digits.is_empty())
        .then_some(digits)
        .and_then(|digits| digits.parse().ok())
}
