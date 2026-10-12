#[test]
fn included_assets_use_a_manifest_path_index() {
    let source = include_str!("../manifest.rs")
        .split_once("#[cfg(test)]")
        .unwrap()
        .0;
    let function = source.split("pub fn pack_inputs").nth(1).unwrap();
    let function = function.split("fn source_path").next().unwrap();

    assert!(function.contains("HashMap::with_capacity(self.assets.len())"));
    assert!(function.contains("entries_by_path.get(path.as_str())"));
    assert!(!function.contains("self.assets.iter().find"));
}
