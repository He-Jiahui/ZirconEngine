mod decode_obj_file;
mod error;
mod obj_vertex_key;
mod parse_obj_face_vertex;
mod parse_obj_scalar;
mod parsed_obj_vertex;
mod resolve_obj_index;

pub(crate) use decode_obj_file::decode_obj_file;
pub(crate) use error::{ObjDecodeError, ObjDecodeResult};
