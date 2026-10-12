// 表切换是静态测试投影的提交边界，离开本类表后不再吸收同名字段。
use super::storage::ModuleContributionParserState;

impl ModuleContributionParserState {
    pub(in super::super) fn module_table_transition_consumed(&mut self, line: &str) -> bool {
        if line == "[[modules]]" {
            self.push_current_module();
            self.inside_module = true;
            return true;
        }
        if line.starts_with("[[") {
            self.push_current_module();
            self.inside_module = false;
        }
        !self.inside_module
    }
}
