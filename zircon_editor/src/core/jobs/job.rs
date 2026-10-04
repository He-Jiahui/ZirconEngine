//! 后台工作只通过统一任务系统取得执行上下文并交还类型化结果；工作和结果必须可跨线程转移且不借用宿主，界面状态交接留在消费端。
use super::{JobContext, JobError};

/// 后台工作的所有权入口；消耗工作载荷并交还结果，宿主状态由票据消费端接续。
pub trait EditorJob: Send + 'static {
    type Output: Send + 'static;

    fn run(self, context: JobContext) -> Result<Self::Output, JobError>;
}
