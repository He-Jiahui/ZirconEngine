//! 将启动实例、非零序号与 nonce 组合为跨请求传递的项目激活操作身份。
//! 进程级调用方复用本模块的生成器；序号耗尽时分配返回 `None`。

#[path = "generator.rs"]
mod generator;
#[path = "operation_id.rs"]
mod operation_id;
#[path = "../operation_id_error.rs"]
mod operation_id_error;
#[path = "sequence.rs"]
mod sequence;
#[path = "sequence_error.rs"]
mod sequence_error;
#[path = "source_instance.rs"]
mod source_instance;
#[path = "../source_instance_error.rs"]
mod source_instance_error;

pub use generator::ProjectActivationOperationIdGenerator;
pub use operation_id::ProjectActivationOperationId;
pub use operation_id_error::ProjectActivationOperationIdError;
pub use sequence::ProjectActivationOperationSequence;
pub use sequence_error::ProjectActivationOperationSequenceError;
pub use source_instance::ProjectLaunchInstanceId;
pub use source_instance_error::ProjectLaunchInstanceIdError;
