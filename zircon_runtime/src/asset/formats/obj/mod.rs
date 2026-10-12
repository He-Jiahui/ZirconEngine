//! 文件式网格加载使用此 OBJ 解码器；它不负责项目导入事务、材质和子资产。

#[path = "decode_obj_file.rs"]
mod decode_obj_file;
#[path = "error.rs"]
mod error;
#[path = "obj_vertex_key.rs"]
mod obj_vertex_key;
#[path = "parse_obj_face_vertex.rs"]
mod parse_obj_face_vertex;
#[path = "parse_obj_scalar.rs"]
mod parse_obj_scalar;
#[path = "parsed_obj_vertex.rs"]
mod parsed_obj_vertex;
#[path = "resolve_obj_index.rs"]
mod resolve_obj_index;

pub(crate) use decode_obj_file::decode_obj_file;
pub(crate) use error::{ObjDecodeError, ObjDecodeResult};
