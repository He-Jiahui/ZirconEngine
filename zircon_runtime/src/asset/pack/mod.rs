//! ZrPack 的 manifest、chunk 去重、delta、读写和安装事务统一从此模块导出。
//! reader/writer 负责 pack 格式与内容；安装子模块负责 installed、backup 和 journal 的事务发布。

mod dedup;
mod delta;
mod install;
mod manifest;
mod reader;
mod trim;
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
