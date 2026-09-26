//! 导航派发的共享契约：处理器通过上下文读取路由、返回决议，结果承接焦点与绑定回执。

mod context;
mod effect;
mod invocation;
mod result;

pub use context::UiNavigationDispatchContext;
pub use effect::UiNavigationDispatchEffect;
pub use invocation::UiNavigationDispatchInvocation;
pub use result::UiNavigationDispatchResult;
