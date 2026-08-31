use std::sync::Arc;

use super::EditorConsoleMessageLevel;

pub(super) const CONSOLE_OUTPUT_LINE_CHUNK_CAPACITY: usize = 64;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
/// Describes the immutable generation's transition from its predecessor.
pub(crate) struct ConsoleOutputLineDelta {
    pub entered: usize,
    pub expired: usize,
    pub retained: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ConsoleOutputLineSnapshot {
    source_id: u64,
    slot_id: u64,
    text: Arc<str>,
    level: EditorConsoleMessageLevel,
    jump_sequence: Option<u64>,
    action_id: Option<Arc<str>>,
}

impl ConsoleOutputLineSnapshot {
    pub(crate) fn new(
        source_id: u64,
        text: Arc<str>,
        level: EditorConsoleMessageLevel,
        jump_sequence: Option<u64>,
        action_id: Option<Arc<str>>,
    ) -> Self {
        Self {
            source_id,
            slot_id: source_id,
            text,
            level,
            jump_sequence,
            action_id,
        }
    }

    pub(crate) fn text(&self) -> &str {
        self.text
            .strip_suffix('\r')
            .unwrap_or_else(|| self.text.as_ref())
    }

    pub(crate) fn raw_text(&self) -> &str {
        self.text.as_ref()
    }

    pub(crate) const fn level(&self) -> EditorConsoleMessageLevel {
        self.level
    }

    pub(crate) const fn jump_sequence(&self) -> Option<u64> {
        self.jump_sequence
    }

    pub(crate) fn action_id(&self) -> Option<&str> {
        self.action_id.as_deref()
    }

    pub(crate) const fn source_id(&self) -> u64 {
        self.source_id
    }

    pub(crate) const fn slot_id(&self) -> u64 {
        self.slot_id
    }

    pub(crate) fn with_slot_id(mut self, slot_id: u64) -> Self {
        self.slot_id = slot_id;
        self
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct ConsoleOutputLineGeneration {
    chunks: Arc<[Arc<[ConsoleOutputLineSnapshot]>]>,
    first_chunk_offset: usize,
    line_count: usize,
}

impl ConsoleOutputLineGeneration {
    pub(crate) fn from_lines(lines: Vec<ConsoleOutputLineSnapshot>) -> Self {
        if lines.is_empty() {
            return Self::default();
        }
        let chunks = lines
            .chunks(CONSOLE_OUTPUT_LINE_CHUNK_CAPACITY)
            .map(Arc::<[ConsoleOutputLineSnapshot]>::from)
            .collect::<Vec<_>>();
        Self {
            chunks: chunks.into(),
            first_chunk_offset: 0,
            line_count: lines.len(),
        }
    }

    pub(crate) fn append_bounded(
        &self,
        entered_lines: Vec<ConsoleOutputLineSnapshot>,
        capacity: usize,
    ) -> (Self, ConsoleOutputLineDelta) {
        if entered_lines.is_empty() || capacity == 0 {
            return (
                if capacity == 0 {
                    Self::default()
                } else {
                    self.clone()
                },
                ConsoleOutputLineDelta {
                    entered: 0,
                    expired: if capacity == 0 { self.line_count } else { 0 },
                    retained: if capacity == 0 { 0 } else { self.line_count },
                },
            );
        }

        let entered_count = entered_lines.len().min(capacity);
        if entered_lines.len() >= capacity {
            let retained_start = entered_lines.len() - capacity;
            return (
                Self::from_lines(entered_lines[retained_start..].to_vec()),
                ConsoleOutputLineDelta {
                    entered: entered_count,
                    expired: self.line_count,
                    retained: 0,
                },
            );
        }

        let old_line_count = self.line_count;
        let expired = old_line_count
            .saturating_add(entered_lines.len())
            .saturating_sub(capacity);
        let mut chunks = self.chunks.to_vec();
        let mut entered = entered_lines.into_iter();

        if let Some(last) = chunks.last_mut() {
            if last.len() < CONSOLE_OUTPUT_LINE_CHUNK_CAPACITY {
                let mut tail = last.to_vec();
                tail.extend(
                    entered
                        .by_ref()
                        .take(CONSOLE_OUTPUT_LINE_CHUNK_CAPACITY.saturating_sub(tail.len())),
                );
                *last = tail.into();
            }
        }
        loop {
            let chunk = entered
                .by_ref()
                .take(CONSOLE_OUTPUT_LINE_CHUNK_CAPACITY)
                .collect::<Vec<_>>();
            if chunk.is_empty() {
                break;
            }
            chunks.push(chunk.into());
        }

        let mut next = Self {
            chunks: chunks.into(),
            first_chunk_offset: self.first_chunk_offset,
            line_count: old_line_count.saturating_add(entered_count),
        };
        next.trim_front(expired);
        (
            next,
            ConsoleOutputLineDelta {
                entered: entered_count,
                expired,
                retained: old_line_count.saturating_sub(expired),
            },
        )
    }

    pub(crate) fn trim_before_source_id(&self, first_source_id: u64) -> (Self, usize) {
        let expired = self
            .iter()
            .take_while(|line| line.source_id() < first_source_id)
            .count();
        if expired == 0 {
            return (self.clone(), 0);
        }
        let mut next = self.clone();
        next.trim_front(expired);
        (next, expired)
    }

    fn trim_front(&mut self, mut count: usize) {
        count = count.min(self.line_count);
        self.line_count -= count;
        if count == 0 {
            return;
        }

        let mut chunks = self.chunks.to_vec();
        let mut offset = self.first_chunk_offset;
        while count > 0 && !chunks.is_empty() {
            let available = chunks[0].len().saturating_sub(offset);
            if count < available {
                offset += count;
                count = 0;
            } else {
                count -= available;
                chunks.remove(0);
                offset = 0;
            }
        }
        if self.line_count == 0 {
            chunks.clear();
            offset = 0;
        }
        self.chunks = chunks.into();
        self.first_chunk_offset = offset;
    }

    pub(crate) const fn len(&self) -> usize {
        self.line_count
    }

    pub(crate) const fn is_empty(&self) -> bool {
        self.line_count == 0
    }

    pub(crate) fn first(&self) -> Option<&ConsoleOutputLineSnapshot> {
        self.get(0)
    }

    pub(crate) fn get(&self, index: usize) -> Option<&ConsoleOutputLineSnapshot> {
        let (chunk, chunk_row) = self.chunk_and_row(index)?;
        chunk.get(chunk_row)
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &ConsoleOutputLineSnapshot> {
        let first_chunk_offset = self.first_chunk_offset;
        self.chunks
            .iter()
            .enumerate()
            .flat_map(move |(chunk_index, chunk)| {
                let start = if chunk_index == 0 {
                    first_chunk_offset.min(chunk.len())
                } else {
                    0
                };
                chunk[start..].iter()
            })
            .take(self.line_count)
    }

    pub(crate) fn shares_storage_chunk_with(
        &self,
        other: &Self,
        self_index: usize,
        other_index: usize,
    ) -> bool {
        self.chunk_and_row(self_index)
            .zip(other.chunk_and_row(other_index))
            .is_some_and(|((left, _), (right, _))| Arc::ptr_eq(left, right))
    }

    fn chunk_and_row(&self, index: usize) -> Option<(&Arc<[ConsoleOutputLineSnapshot]>, usize)> {
        if index >= self.line_count {
            return None;
        }
        let mut remaining = index.saturating_add(self.first_chunk_offset);
        for chunk in self.chunks.iter() {
            if remaining < chunk.len() {
                return Some((chunk, remaining));
            }
            remaining -= chunk.len();
        }
        None
    }
}

impl PartialEq for ConsoleOutputLineGeneration {
    fn eq(&self, other: &Self) -> bool {
        self.line_count == other.line_count && self.iter().eq(other.iter())
    }
}

impl Eq for ConsoleOutputLineGeneration {}
