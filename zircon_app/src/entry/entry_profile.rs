/// 入口层选择编辑器、交互运行或无窗口执行的粗粒度策略。
/// 产品角色先映射到此策略，再由解析后的目标模式和能力约束决定实际宿主。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntryProfile {
    Editor,
    Runtime,
    Headless,
}
