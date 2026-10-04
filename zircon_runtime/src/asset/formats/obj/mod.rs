//! 文件式网格加载使用此 OBJ 解码器；它不负责项目导入事务、材质和子资产。

mod decode_obj_file;
mod error;
mod obj_vertex_key;
mod parse_obj_face_vertex;
mod parse_obj_scalar;
mod parsed_obj_vertex;
mod resolve_obj_index;

pub(crate) use decode_obj_file::decode_obj_file;
pub(crate) use error::{ObjDecodeError, ObjDecodeResult};
