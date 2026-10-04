#[inline]
pub(super) fn collect_fixed_packed_words<T, const WORD_COUNT: usize>(
    values: &[T],
    packed_words: impl Fn(&T) -> [u32; WORD_COUNT],
) -> Vec<u32> {
    let capacity = values
        .len()
        .checked_mul(WORD_COUNT)
        .expect("fixed-width packed word count exceeds addressable memory");
    let mut words = Vec::with_capacity(capacity);
    for value in values {
        words.extend_from_slice(&packed_words(value));
    }
    words
}

#[cfg(test)]
#[path = "tests/packed_words.rs"]
mod tests;
