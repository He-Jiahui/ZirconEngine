use std::fs;

use super::{
    DurableJournal, DurableJournalEntry, DurableJournalError, JournalDocumentKey,
    JournalReadReport, JournalTailFault, MAX_JOURNAL_BYTES, MAX_JOURNAL_RECORDS, MAX_RECORD_BYTES,
    TransactionJournal, little_endian_u32, little_endian_u64, read_header, take,
};

const MIN_JOURNAL_FRAME_BYTES: usize =
    std::mem::size_of::<u64>() + std::mem::size_of::<u32>() + blake3::OUT_LEN;

impl DurableJournal {
    pub fn read(
        &self,
        document: &JournalDocumentKey,
    ) -> Result<JournalReadReport, DurableJournalError> {
        let path = self.path_for(document);
        let bytes = read_bounded_file(&path)?;
        let mut cursor = 0;
        let header = read_header(&bytes, &mut cursor, &path)?;
        if header.document_key != document.as_str()
            || header.source_path.as_path() != document.source_path()
        {
            return Err(DurableJournalError::DocumentMismatch { path });
        }

        let mut entries = Vec::with_capacity(journal_entry_capacity(bytes.len(), cursor));
        let mut tail_fault = None;
        let mut expected_sequence = match header.base_sequence.checked_add(1) {
            Some(sequence) => sequence,
            None => {
                return Err(DurableJournalError::InvalidHeader { path });
            }
        };
        while cursor < bytes.len() {
            if entries.len() == MAX_JOURNAL_RECORDS {
                tail_fault = Some(JournalTailFault::RecordLimitExceeded);
                break;
            }
            let Some(sequence_bytes) = take(&bytes, &mut cursor, 8) else {
                tail_fault = Some(JournalTailFault::TruncatedFrame);
                break;
            };
            let Some(sequence) = little_endian_u64(sequence_bytes) else {
                tail_fault = Some(JournalTailFault::TruncatedFrame);
                break;
            };
            let Some(length_bytes) = take(&bytes, &mut cursor, 4) else {
                tail_fault = Some(JournalTailFault::TruncatedFrame);
                break;
            };
            let Some(length) = little_endian_u32(length_bytes) else {
                tail_fault = Some(JournalTailFault::TruncatedFrame);
                break;
            };
            let length = length as usize;
            if length > MAX_RECORD_BYTES {
                tail_fault = Some(JournalTailFault::RecordTooLarge { sequence, length });
                break;
            }
            let Some(expected_digest) = take(&bytes, &mut cursor, blake3::OUT_LEN) else {
                tail_fault = Some(JournalTailFault::TruncatedFrame);
                break;
            };
            let Some(payload) = take(&bytes, &mut cursor, length) else {
                tail_fault = Some(JournalTailFault::TruncatedFrame);
                break;
            };
            if blake3::hash(payload).as_bytes() != expected_digest {
                tail_fault = Some(JournalTailFault::ChecksumMismatch { sequence });
                break;
            }
            let transaction = match TransactionJournal::decode(payload) {
                Ok(transaction) => transaction,
                Err(source) => {
                    tail_fault = Some(JournalTailFault::InvalidTransaction { sequence, source });
                    break;
                }
            };
            if sequence != expected_sequence {
                tail_fault = Some(JournalTailFault::NonContiguousSequence { sequence });
                break;
            }
            entries.push(DurableJournalEntry {
                sequence,
                transaction,
            });
            let Some(next_sequence) = sequence.checked_add(1) else {
                tail_fault = Some(JournalTailFault::NonContiguousSequence { sequence });
                break;
            };
            expected_sequence = next_sequence;
        }
        Ok(JournalReadReport {
            base_sequence: header.base_sequence,
            entries,
            tail_fault,
        })
    }
}

pub(super) fn document_key_from_journal_path(
    path: &std::path::Path,
) -> Result<JournalDocumentKey, DurableJournalError> {
    let bytes = read_bounded_file(path)?;
    let mut cursor = 0;
    let header = read_header(&bytes, &mut cursor, path)?;
    let document =
        JournalDocumentKey::from_project_relative_path(&header.source_path).map_err(|_| {
            DurableJournalError::InvalidHeader {
                path: path.to_path_buf(),
            }
        })?;
    if header.document_key != document.as_str() {
        return Err(DurableJournalError::InvalidHeader {
            path: path.to_path_buf(),
        });
    }
    Ok(document)
}

fn journal_entry_capacity(encoded_len: usize, payload_offset: usize) -> usize {
    (encoded_len.saturating_sub(payload_offset) / MIN_JOURNAL_FRAME_BYTES).min(MAX_JOURNAL_RECORDS)
}

fn read_bounded_file(path: &std::path::Path) -> Result<Vec<u8>, DurableJournalError> {
    let metadata = fs::symlink_metadata(path).map_err(|source| DurableJournalError::Io {
        operation: "inspect journal",
        path: path.to_path_buf(),
        source,
    })?;
    if !metadata.file_type().is_file() {
        return Err(DurableJournalError::UnexpectedFileType {
            path: path.to_path_buf(),
        });
    }
    if metadata.len() > MAX_JOURNAL_BYTES {
        return Err(DurableJournalError::FileTooLarge {
            path: path.to_path_buf(),
            bytes: metadata.len(),
            maximum: MAX_JOURNAL_BYTES,
        });
    }
    fs::read(path).map_err(|source| DurableJournalError::Io {
        operation: "read journal",
        path: path.to_path_buf(),
        source,
    })
}

#[cfg(test)]
mod optimization_batch_20260830co_editor_tests {
    use super::*;

    #[test]
    fn optimization_batch_20260830co_editor_journal_capacity_is_bounded_by_frames_and_limit() {
        const PAYLOAD_OFFSET: usize = 73;

        assert_eq!(journal_entry_capacity(12, PAYLOAD_OFFSET), 0);
        assert_eq!(
            journal_entry_capacity(PAYLOAD_OFFSET + MIN_JOURNAL_FRAME_BYTES * 2, PAYLOAD_OFFSET),
            2
        );
        assert_eq!(
            journal_entry_capacity(
                PAYLOAD_OFFSET + MIN_JOURNAL_FRAME_BYTES * (MAX_JOURNAL_RECORDS + 1),
                PAYLOAD_OFFSET
            ),
            MAX_JOURNAL_RECORDS
        );
    }

    #[test]
    #[ignore = "managed Windows release performance evidence"]
    fn optimization_batch_20260830co_editor_journal_entry_capacity_evidence() {
        let encoded_len = MIN_JOURNAL_FRAME_BYTES * MAX_JOURNAL_RECORDS;
        let legacy_growth_events = collect_growth_events(0);
        let optimized_growth_events = collect_growth_events(journal_entry_capacity(encoded_len, 0));

        println!(
            "EDITOR502_JOURNAL_READ_ENTRY_CAPACITY_BENCH_V1 records={MAX_JOURNAL_RECORDS} \
legacy_growth_events={legacy_growth_events} optimized_growth_events={optimized_growth_events} \
growth_event_reduction_pct=100"
        );
        assert!(legacy_growth_events > 0);
        assert_eq!(optimized_growth_events, 0);
    }

    fn collect_growth_events(capacity: usize) -> usize {
        let mut entries = Vec::with_capacity(capacity);
        let mut growth_events = 0;
        for entry in 0..MAX_JOURNAL_RECORDS {
            let previous_capacity = entries.capacity();
            entries.push(entry);
            growth_events += usize::from(entries.capacity() != previous_capacity);
        }
        std::hint::black_box(entries);
        growth_events
    }
}
