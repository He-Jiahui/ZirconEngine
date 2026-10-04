use super::*;

fn test_key() -> SdfAtlasGlyphKey {
    SdfAtlasGlyphKey {
        glyph: 'A',
        glyph_id: Some(1),
        font_id: None,
        font_instance_id: None,
        font: Some("res://fonts/default.font.toml".into()),
        font_family: Some("Zircon Sans".into()),
        language: None,
        font_weight: 400,
        bake_params: SdfBakeParams::default(),
    }
}

#[test]
fn worker_panic_releases_pending_key_and_marks_the_face_terminal() {
    let key = test_key();
    let face = FontFaceId(9);
    let work_id = SdfGenerationWorkId::new(3, 5);
    let mut state = SdfAsyncGenerationState::default();
    state.pending_keys.insert(key.clone());
    state.pending_batches.insert(
        work_id,
        vec![AsyncBatchEntry {
            key: key.clone(),
            face,
            glyph_id: 1,
        }],
    );

    state.finish_pending_batch_with_failure(work_id, SdfGlyphGenerationError::WorkerPanic);

    assert!(!state.pending_keys.contains(&key));
    assert!(state.pending_batches.is_empty());
    assert_eq!(
        state.face_failures.get(&FaceFailureKey { key, face }),
        Some(&SdfGlyphGenerationError::WorkerPanic)
    );
}

#[test]
fn moved_async_batches_preserve_boundaries_and_entry_order() {
    fn glyph_ids(entries: Vec<AsyncBatchEntry>) -> Vec<u16> {
        entries.into_iter().map(|entry| entry.glyph_id).collect()
    }

    let mut entries = (1..=5)
        .map(|glyph_id| AsyncBatchEntry {
            key: test_key(),
            face: FontFaceId(9),
            glyph_id,
        })
        .collect::<Vec<_>>()
        .into_iter();

    let first = take_async_batch(&mut entries, 2);
    let second = take_async_batch(&mut entries, 2);
    let third = take_async_batch(&mut entries, 2);
    assert_eq!(glyph_ids(first), vec![1, 2]);
    assert_eq!(glyph_ids(second), vec![3, 4]);
    assert_eq!(glyph_ids(third), vec![5]);
    assert!(take_async_batch(&mut entries, 2).is_empty());
}
