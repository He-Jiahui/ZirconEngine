//! 交互提取模块将身份、值产品和渲染发布协议一起提供给控制器，指针调用方必须处理未准备状态。

mod cache;
mod extract;
mod key;
#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;

pub(in crate::scene::viewport) use cache::{
    ViewportInteractionExtractCache, ViewportInteractionExtractPointerResolution,
};
pub(in crate::scene::viewport) use extract::ViewportInteractionExtract;
