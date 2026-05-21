pub(crate) fn assert_source_order(source: &str, needles: &[&str], message: &str) {
    let mut offset = 0;
    for needle in needles {
        let Some(index) = source[offset..].find(needle) else {
            panic!("{message}: missing `{needle}`");
        };
        offset += index + needle.len();
    }
}
