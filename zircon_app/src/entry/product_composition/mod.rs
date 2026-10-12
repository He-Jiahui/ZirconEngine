//! 装配请求与已启动产品组合的边界；先准备一致的选择回执，再让组合结果持有运行期 owner。

#[path = "composition.rs"]
mod composition;
#[path = "ownership.rs"]
pub(in crate::entry) mod ownership;
#[path = "request.rs"]
mod request;

pub use composition::ProductComposition;
pub use request::ProductCompositionRequest;
