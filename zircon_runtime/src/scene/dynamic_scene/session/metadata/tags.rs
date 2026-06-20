pub(super) fn normalize_metadata_tags(tags: &mut Vec<String>) {
    for tag in tags.iter_mut() {
        *tag = tag.trim().to_string();
    }
    tags.retain(|tag| !tag.is_empty());
    tags.sort();
    tags.dedup();
}
