use super::obj_vertex_key::ObjVertexKey;
use super::resolve_obj_index::resolve_obj_index;

pub(super) fn parse_obj_face_vertex(
    token: &str,
    position_count: usize,
    uv_count: usize,
    normal_count: usize,
) -> Result<ObjVertexKey, String> {
    let mut parts = token.split('/');
    let position = resolve_obj_index(
        parts.next().unwrap_or_default(),
        position_count,
        "position index",
    )?;
    let uv = match parts.next() {
        Some("") | None => None,
        Some(value) => Some(resolve_obj_index(value, uv_count, "uv index")?),
    };
    let normal = match parts.next() {
        Some("") | None => None,
        Some(value) => Some(resolve_obj_index(value, normal_count, "normal index")?),
    };

    Ok(ObjVertexKey {
        position,
        uv,
        normal,
    })
}
