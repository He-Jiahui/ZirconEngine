// 有身份字段的静态行才纳入比较；必填字段缺失时应使测试失败，防止不完整声明被接受。
use super::super::storage::DependencyParserState;
use super::required;

impl DependencyParserState {
    pub(in super::super::super) fn push_current_dependency(&mut self) {
        let Some(id) = self.current_id.take() else {
            return;
        };
        self.dependencies.push((
            id,
            required::take_required_dependency_required(&mut self.current_required),
            self.current_capability.take(),
        ));
    }
}
