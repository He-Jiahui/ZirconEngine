//! 比例链接资源沿用旧的连接层相对顺序，基础 order 来自 node pipeline；上游仍需保留一层 i32 余量。

const SCALE_LINK_CONNECTOR_OFFSET: i32 = 1;

pub(super) fn scale_link_connector_order(lobe_order: i32) -> i32 {
    lobe_order + SCALE_LINK_CONNECTOR_OFFSET
}

#[cfg(test)]
#[path = "tests/layers.rs"]
mod tests;
