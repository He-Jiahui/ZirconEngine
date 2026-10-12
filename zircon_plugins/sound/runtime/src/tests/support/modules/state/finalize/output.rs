// 文件结束时补交最后一行，保证末尾没有下一张表时仍纳入静态对照。
use super::super::super::super::StaticModule;
use super::super::storage::ModuleContributionParserState;

impl ModuleContributionParserState {
    pub(in super::super::super) fn finish(mut self) -> Vec<StaticModule> {
        self.push_current_module();
        self.modules
    }
}
