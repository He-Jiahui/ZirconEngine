use serde::{Deserialize, Serialize};

/// 排序后绘制元素序列中的连续区间；不能直接用作原始提取输入的下标。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiBatchRange {
    pub first_element: usize,
    pub element_count: usize,
}
