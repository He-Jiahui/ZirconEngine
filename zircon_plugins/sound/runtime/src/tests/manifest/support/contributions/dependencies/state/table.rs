// 表切换是静态测试投影的提交边界，离开本类表后不再吸收同名字段。
use super::storage::DependencyParserState;

impl DependencyParserState {
    pub(in super::super) fn begin_dependency_table(&mut self) {
        self.push_current_dependency();
        self.inside_dependency = true;
    }

    pub(in super::super) fn leave_dependency_table(&mut self) {
        self.push_current_dependency();
        self.inside_dependency = false;
    }

    pub(in super::super) fn is_inside_dependency(&self) -> bool {
        self.inside_dependency
    }
}
