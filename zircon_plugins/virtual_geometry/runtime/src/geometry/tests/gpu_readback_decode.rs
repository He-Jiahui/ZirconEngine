use super::decode::{completed_page_assignments, page_table_entries};

fn little_endian_words(words: &[u32]) -> Vec<u8> {
    words.iter().flat_map(|word| word.to_le_bytes()).collect()
}

#[test]
fn gpu_completion_decode_preserves_assignments_and_recycled_page_ids() {
    let bytes = little_endian_words(&[2, 100, 1, u32::MAX, 200, 2, 42, 999]);

    let decoded = completed_page_assignments(&bytes, 8).unwrap();

    assert_eq!(decoded.0, vec![(100, 1), (200, 2)]);
    assert_eq!(decoded.1, vec![100, 200]);
    assert_eq!(decoded.2, vec![(200, 42)]);
}

#[test]
fn gpu_completion_decode_ignores_incomplete_records_and_trailing_words() {
    let bytes = little_endian_words(&[3, 100, 1, u32::MAX, 200, 2, 42, 300, 3]);

    let decoded = completed_page_assignments(&bytes, 9).unwrap();

    assert_eq!(decoded.0, vec![(100, 1), (200, 2)]);
    assert_eq!(decoded.1, vec![100, 200]);
    assert_eq!(decoded.2, vec![(200, 42)]);
}

#[test]
fn gpu_page_table_decode_keeps_reused_slots_inside_the_resident_prefix() {
    let bytes = little_endian_words(&[30, 0, 20, 1, 40, 2, 50, 3]);

    let entries = page_table_entries(&bytes, 8, 2, vec![0, 1], &[(30, 0), (40, 2)]).unwrap();

    assert_eq!(entries, vec![(30, 0), (20, 1), (40, 2)]);
}

#[test]
fn gpu_readback_decode_rejects_truncated_word_buffers() {
    let short_bytes = little_endian_words(&[1, 100, 1]);

    assert!(completed_page_assignments(&short_bytes, 4).is_err());
    assert!(page_table_entries(&short_bytes, 4, 1, vec![0], &[]).is_err());
}
