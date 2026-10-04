// 进入另一子表前先完成未提交的依赖和模块，确保一个功能内多行各自独立。
use super::super::{flush, OptionalFeatureParserState};

pub(super) fn flush_dependency_and_module_rows(state: &mut OptionalFeatureParserState) {
    flush::flush_pending_dependency(state);
    flush::flush_pending_module(state);
}
