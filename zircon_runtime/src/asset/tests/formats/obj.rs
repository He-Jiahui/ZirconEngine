use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::core::math::Vec3;

use crate::asset::formats::obj::{decode_obj_file, ObjDecodeError};
use crate::asset::tests::project::unique_temp_project_root;

#[test]
fn obj_mesh_file_is_parsed_into_gpu_ready_payload() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("zircon_asset_mesh_{unique}.obj"));
    fs::write(
        &path,
        "\
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 1.0 1.0 0.0
v 0.0 1.0 0.0
vt 0.0 0.0
vt 1.0 0.0
vt 1.0 1.0
vt 0.0 1.0
f 1/1 2/2 3/3 4/4
",
    )
    .unwrap();

    let payload = decode_obj_file(path.to_str().unwrap()).unwrap();
    let _ = fs::remove_file(&path);

    assert_eq!(payload.indices.len(), 6);
    assert_eq!(payload.vertices.len(), 4);
    assert!(payload
        .vertices
        .iter()
        .all(|vertex| Vec3::from_array(vertex.normal).length() > 0.0));
}

#[test]
fn obj_decode_reports_typed_read_error_source() {
    let root = unique_temp_project_root("obj_decode_missing_file");
    let missing = root.join("missing.obj");
    let path = missing.to_string_lossy().to_string();

    let error = decode_obj_file(&path).expect_err("missing OBJ should fail");

    match error {
        ObjDecodeError::Read {
            path: actual_path,
            source,
        } => {
            assert_eq!(actual_path, path);
            assert_eq!(source.kind(), std::io::ErrorKind::NotFound);
        }
        other => panic!("unexpected OBJ error: {other:?}"),
    }

    let _ = fs::remove_dir_all(root);
}

#[test]
fn obj_decode_reports_typed_scalar_parse_error_source() {
    let root = unique_temp_project_root("obj_decode_invalid_scalar");
    fs::create_dir_all(&root).unwrap();
    let path = root.join("invalid.obj");
    fs::write(&path, "v nope 0.0 0.0\n").unwrap();
    let path_string = path.to_string_lossy().to_string();

    let error = decode_obj_file(&path_string).expect_err("invalid OBJ scalar should fail");

    match error {
        ObjDecodeError::InvalidScalar {
            path: actual_path,
            line,
            label,
            value,
            source,
        } => {
            assert_eq!(actual_path, path_string);
            assert_eq!(line, 1);
            assert_eq!(label, "vertex x");
            assert_eq!(value, "nope");
            assert!(source.to_string().contains("invalid"));
        }
        other => panic!("unexpected OBJ error: {other:?}"),
    }

    let _ = fs::remove_dir_all(root);
}

#[test]
fn obj_face_parser_does_not_collect_a_second_token_buffer() {
    let source = include_str!("../../formats/obj/decode_obj_file.rs");

    assert!(!source.contains("let tokens: Vec<_> = parts.collect()"));
    assert!(source.contains(".into_iter().chain(parts)"));
}
