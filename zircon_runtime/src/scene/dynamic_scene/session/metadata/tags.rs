pub(super) fn normalize_metadata_tags(tags: &mut Vec<String>) {
    for tag in tags.iter_mut() {
        trim_tag_in_place(tag);
    }
    tags.retain(|tag| !tag.is_empty());
    tags.sort();
    tags.dedup();
}

fn trim_tag_in_place(tag: &mut String) {
    let trimmed_end = tag.trim_end().len();
    tag.truncate(trimmed_end);

    let trimmed_start = tag.len() - tag.trim_start().len();
    if trimmed_start != 0 {
        tag.drain(..trimmed_start);
    }
}

#[cfg(test)]
#[path = "tags/in_place_tests.rs"]
mod in_place_tests;
