use super::*;

#[test]
fn single_pass_fuzzy_score_preserves_exact_and_subsequence_ranking() {
    let exact_query = EditorCommandPaletteCompiledQuery::new("ha");
    let mut exact_metrics = EditorCommandPaletteQueryMetrics::default();
    let exact = fuzzy_score("alpha beta", &exact_query, &mut exact_metrics);

    let subsequence_query = EditorCommandPaletteCompiledQuery::new("ab");
    let mut subsequence_metrics = EditorCommandPaletteQueryMetrics::default();
    let subsequence = fuzzy_score("alpha beta", &subsequence_query, &mut subsequence_metrics);

    assert_eq!(exact, Some(255));
    assert_eq!(subsequence, Some(219));
    assert!(exact_metrics.document_byte_visits <= "alpha beta".len());
    assert_eq!(subsequence_metrics.document_byte_visits, "alpha beta".len());
}

#[test]
fn later_exact_match_still_overrides_an_earlier_subsequence() {
    let query = EditorCommandPaletteCompiledQuery::new("ab");
    let mut metrics = EditorCommandPaletteQueryMetrics::default();

    assert_eq!(
        fuzzy_score("a gap before ab", &query, &mut metrics),
        Some(255)
    );
    assert!(metrics.document_byte_visits < "a gap before ab".len());
}

#[test]
fn rarest_posting_keeps_repeated_query_byte_candidates() {
    let query = EditorCommandPaletteCompiledQuery::new("letter");
    let mut postings: [Box<[usize]>; 256] = std::array::from_fn(|_| Vec::new().into_boxed_slice());
    postings[usize::from(b'e')] = vec![2, 7].into_boxed_slice();
    postings[usize::from(b'l')] = vec![2, 4, 7].into_boxed_slice();
    postings[usize::from(b't')] = vec![2, 5, 7].into_boxed_slice();
    postings[usize::from(b'r')] = vec![2, 6, 7].into_boxed_slice();

    assert_eq!(query.rarest_posting(&postings), &[2, 7]);
}
