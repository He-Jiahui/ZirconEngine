use std::path::Path;

use super::{same_source_engine_path, source_engine_id};

#[test]
fn source_engine_paths_share_project_filesystem_key_normalization() {
    assert!(same_source_engine_path(
        Path::new("E:\\Git\\ZirconEngine\\"),
        Path::new("e:/git/zirconengine")
    ));
    assert_eq!(
        source_engine_id(Path::new("E:\\Git\\ZirconEngine\\")),
        source_engine_id(Path::new("e:/git/zirconengine"))
    );
}
