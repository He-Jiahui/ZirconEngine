//! 列表行以状态底面为起点依次放选择指示、标题和尾部装饰；上游须保留局部偏移余量。

const SELECTION_INDICATOR_OFFSET: i32 = 1;
const LABEL_OFFSET: i32 = 2;
const ADORNMENT_OFFSET: i32 = 3;

pub(super) fn selection_indicator_order(surface_order: i32) -> i32 {
    surface_order + SELECTION_INDICATOR_OFFSET
}

pub(super) fn label_order(surface_order: i32) -> i32 {
    surface_order + LABEL_OFFSET
}

pub(super) fn adornment_order(surface_order: i32) -> i32 {
    surface_order + ADORNMENT_OFFSET
}

#[cfg(test)]
#[path = "tests/layers.rs"]
mod tests;
