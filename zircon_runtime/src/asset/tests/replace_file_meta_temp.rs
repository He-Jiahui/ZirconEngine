use std::path::Path;

use super::is_replace_file_meta_temporary;

#[test]
fn recognizes_only_the_observed_replace_file_zmeta_temp_shape() {
    for path in [
        "pbr_shader.zmeta~RF63fc050.TMP",
        "cube.obj.zmeta~RF640bc83.TMP",
        "bundle.zmeta~RFABCDEF0.TMP",
    ] {
        assert!(
            is_replace_file_meta_temporary(Path::new(path)),
            "expected canonical ReplaceFile metadata temp: {path}"
        );
    }

    for path in [
        "pbr_shader.zmeta~RF63fc05.TMP",
        "pbr_shader.zmeta~RF63fc0500.TMP",
        "pbr_shader.zmeta~RF63fc05g.TMP",
        "pbr_shader.zmeta~rf63fc050.TMP",
        "pbr_shader.zmeta~RF63fc050.tmp",
        "pbr_shader.TMP",
        "pbr_shader.zmeta~RF63fc050.TMP.backup",
    ] {
        assert!(
            !is_replace_file_meta_temporary(Path::new(path)),
            "must retain non-canonical user path: {path}"
        );
    }
}
