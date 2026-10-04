#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// 宿主传入 hook 与回执的保存意图；关闭、批量和显式保存可分别处理。
pub enum SaveReason {
    Explicit,
    SaveAll,
    Close,
}
