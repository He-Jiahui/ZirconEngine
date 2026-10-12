//! 本组测试挂接相同运行时边界的夹具和行为用例；具体限制由子模块的入口断言界定。
#[path = "playback/gameplay_emission.rs"]
mod gameplay_emission;
#[path = "playback/range.rs"]
mod range;
#[path = "playback/settings.rs"]
mod settings;
#[path = "playback/source_completion.rs"]
mod source_completion;
