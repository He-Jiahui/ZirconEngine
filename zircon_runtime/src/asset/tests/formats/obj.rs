use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::core::math::Vec3;

use crate::asset::formats::obj::decode_obj_file;

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
