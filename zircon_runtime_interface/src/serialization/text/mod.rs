//! 版本化文本格式的内部实现，负责封套结构、读取识别与规范化写出。
//! 未封套输入是否按旧版数据接纳，由父级加载入口的显式策略决定。

#[path = "canonical.rs"]
pub(super) mod canonical;
#[path = "canonical_map_key.rs"]
pub(super) mod canonical_map_key;
#[path = "canonical_spool.rs"]
pub(super) mod canonical_spool;
#[path = "canonical_writer.rs"]
pub(super) mod canonical_writer;
#[path = "document.rs"]
pub(super) mod document;
#[path = "envelope.rs"]
pub(super) mod envelope;
#[path = "read.rs"]
pub(super) mod read;
#[path = "wire.rs"]
pub(super) mod wire;
