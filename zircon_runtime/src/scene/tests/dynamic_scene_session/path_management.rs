//! 磁盘归档的单槽导入、复制与变更从路径入口分组验证；
//! 预览供调用方决定操作，原子替换负责发布完整结果。

use std::fs;

use crate::scene::{
    RuntimeSessionArchive, RuntimeSessionArchiveMergePolicy, RuntimeSessionMetadata, World,
};

use super::{tagged_slot, temporary_archive_leftovers, unique_temp_root};

mod archive_merge;
mod mutation_previews;
mod single_slot_import;
mod single_slot_save;
mod slot_copy;
mod slot_mutations;
