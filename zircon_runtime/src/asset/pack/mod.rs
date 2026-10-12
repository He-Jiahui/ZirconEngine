//! ZrPack 的 manifest、chunk 去重、delta、读写和安装事务统一从此模块导出。
//! reader/writer 负责 pack 格式与内容；安装子模块负责 installed、backup 和 journal 的事务发布。

#[path = "dedup.rs"]
mod dedup;
#[path = "delta.rs"]
mod delta;
#[path = "install/mod.rs"]
mod install;
#[path = "manifest.rs"]
mod manifest;
#[path = "reader.rs"]
mod reader;
#[path = "trim.rs"]
mod trim;
#[path = "writer.rs"]
mod writer;

pub use dedup::{zrpack_content_hash, ZrPackDedupTable};
pub use delta::{
    ZrPackDeltaDocumentManifest, ZrPackDeltaReader, ZrPackDeltaWriteReport, ZrPackDeltaWriter,
    ZRPACK_DELTA_MAGIC,
};
pub use install::{
    ZrPackDeltaInstallError, ZrPackDeltaInstallReport, ZrPackDeltaInstaller, ZrPackInstallReceipt,
    ZrPackPromotionMethod, ZrPackPromotionReport, ZRPACK_INSTALL_RECEIPT_FORMAT_VERSION,
};
pub use manifest::{
    ZrChunkEntry, ZrPackAssetEntry, ZrPackDocumentManifest, ZrPackError, ZrPackManifest,
    ZRPACK_FORMAT_VERSION, ZRPACK_MAGIC,
};
pub use reader::ZrPackReader;
pub use trim::{
    ZrPackMissingDependency, ZrPackTrimConfig, ZrPackTrimInputAsset, ZrPackTrimPlanner,
    ZrPackTrimReason, ZrPackTrimReport, ZrPackTrimmedAsset,
};
pub use writer::{ZrPackInputAsset, ZrPackWriteReport, ZrPackWriter};
