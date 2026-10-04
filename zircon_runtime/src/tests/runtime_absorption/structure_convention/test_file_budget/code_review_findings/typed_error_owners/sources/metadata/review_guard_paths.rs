//! 为类型化错误审查读取或聚合父子源码，保留路径与文本的配对关系供上层检查；聚合结果只描述被列入清单的文件。
use super::super::super::super::super::*;

pub(in super::super) const TYPED_ERROR_CHILD_OWNER_LINE_BUDGET: usize = 800;

pub(in super::super) fn typed_error_source_inventory_status_rows_source() -> String {
    let mut source = String::new();
    source
}
