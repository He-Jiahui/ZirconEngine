pub(super) fn source_blob_from(sources: Vec<(&'static str, String)>) -> String {
    let mut blob = String::new();
    for (_, source) in sources {
        blob.push_str(&source);
        blob.push('\n');
    }
    blob
}
