pub(super) fn page_table_words(
    resident_entries: &[[u32; 2]],
    page_table_word_count: usize,
) -> Vec<u32> {
    let mut page_table_words = vec![0_u32; page_table_word_count];
    for (entry_index, [page_id, slot]) in resident_entries.iter().enumerate() {
        let word_index = entry_index * 2;
        page_table_words[word_index] = *page_id;
        page_table_words[word_index + 1] = *slot;
    }
    page_table_words
}
