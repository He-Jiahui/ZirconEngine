/// Caller-owned ceiling for one serialization attempt.
///
/// The budget is intentionally explicit and has no `Default` implementation:
/// each archive owner must choose a limit appropriate for its wire contract.
/// 供一次写入调用使用的输出字节上限，由调用方按线格式契约显式提供。
/// write_canonical_text_to 按输出字节计量（含末尾换行）；该入口不写版本封套，也不代表总内存预算。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SerializationBudget {
    max_output_bytes: usize,
}

impl SerializationBudget {
    /// 创建一个调用方选择的输出字节上限。
    ///
    /// # 参数
    /// - max_output_bytes：单次写入最多允许输出的字节数。
    ///
    /// # 返回值
    /// 返回保存该上限的预算值，由接收它的写入入口用于限制该次输出。
    /// 输出可达到上限；累计输出字节数超出上限时，预算检查会失败。
    pub const fn new(max_output_bytes: usize) -> Self {
        Self { max_output_bytes }
    }

    /// 返回此预算保存的单次输出上限，单位为字节。
    pub const fn max_output_bytes(self) -> usize {
        self.max_output_bytes
    }
}
