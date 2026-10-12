//! pack 增量安装按 staging、promotion、恢复 journal 和 receipt 四个阶段组织；模块外只暴露事务门面及结果类型。

#[path = "delta_workflow.rs"]
mod delta_workflow;
#[path = "error.rs"]
mod error;
#[path = "file_io.rs"]
mod file_io;
#[path = "installer.rs"]
mod installer;
#[path = "promotion.rs"]
mod promotion;
#[path = "promotion_journal.rs"]
mod promotion_journal;
#[path = "promotion_report.rs"]
mod promotion_report;
#[path = "receipt.rs"]
mod receipt;
#[path = "receipt_io.rs"]
mod receipt_io;
#[path = "staging.rs"]
mod staging;
#[path = "staging_report.rs"]
mod staging_report;

pub use error::ZrPackDeltaInstallError;
pub use installer::ZrPackDeltaInstaller;
pub use promotion_report::{ZrPackPromotionMethod, ZrPackPromotionReport};
pub use receipt::{ZrPackInstallReceipt, ZRPACK_INSTALL_RECEIPT_FORMAT_VERSION};
pub use staging_report::ZrPackDeltaInstallReport;
