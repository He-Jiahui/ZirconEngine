//! 维护模型拥有型数据、基础结构验证和 ZRNN 二进制编解码。
//! 此模块处理内存模型与字节切片，不负责文件 I/O 或后端执行。

mod asset;
mod format;
mod validate;

pub use asset::{NnDataType, NnModelAsset, NnTensorDesc, NnTensorKind};
pub use format::NnModelFormatError;
pub use validate::NnModelValidationError;
