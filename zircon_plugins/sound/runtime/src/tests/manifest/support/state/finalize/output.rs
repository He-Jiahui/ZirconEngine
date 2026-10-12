// 文件结束时补交最后一行，保证末尾没有下一张表时仍纳入静态对照。
use super::super::super::super::StaticDependency;
use super::super::storage::DependencyParserState;

impl DependencyParserState {
    pub(in super::super::super) fn finish(mut self) -> Vec<StaticDependency> {
        self.push_current_dependency();
        self.dependencies
    }
}
