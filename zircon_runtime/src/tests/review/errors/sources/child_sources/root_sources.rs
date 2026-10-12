//! 为类型化错误审查读取或聚合父子源码，保留路径与文本的配对关系供上层检查；聚合结果只描述被列入清单的文件。
use super::super::super::super::super::*;
use super::super::*;

pub(in super::super) struct TypedErrorSourceInventorySources {
    pub(in super::super) structure_child: String,
    pub(in super::super) source_inventory_child: String,
    pub(in super::super) paths_child: String,
    pub(in super::super) reads_child: String,
    pub(in super::super) budgets_child: String,
    pub(in super::super) delegation_child: String,
}

pub(in super::super) fn typed_error_source_inventory_sources() -> TypedErrorSourceInventorySources {
    TypedErrorSourceInventorySources {
        structure_child: read_runtime_src(TYPED_ERROR_STRUCTURE_CHILD),
        source_inventory_child: read_runtime_src(TYPED_ERROR_SOURCE_INVENTORY_CHILD),
        paths_child: read_runtime_src(TYPED_ERROR_SOURCE_INVENTORY_PATHS_CHILD),
        reads_child: read_runtime_src(TYPED_ERROR_SOURCE_INVENTORY_READS_CHILD),
        budgets_child: read_runtime_src(TYPED_ERROR_SOURCE_INVENTORY_BUDGETS_CHILD),
        delegation_child: read_runtime_src(TYPED_ERROR_SOURCE_INVENTORY_DELEGATION_CHILD),
    }
}
