use std::sync::Arc;

use crate::asset::AssetUri;
use crate::core::resource::ResourceRecord;

/// The observable result of one Runtime project import transaction.
///
/// A receipt exists only after source files, artifacts, metadata, registry state, and resources
/// were committed under the same active project generation.
///
/// Editor 将该回执作为模型导入 job 的成功输出，并记录来源 URI、项目代序号和已提交记录数。
#[derive(Clone, Debug)]
pub struct ProjectImportReceipt {
    source_uri: AssetUri,
    generation_sequence: u64,
    committed_records: Arc<[ResourceRecord]>,
}

impl ProjectImportReceipt {
    pub(crate) fn new(
        source_uri: AssetUri,
        generation_sequence: u64,
        committed_records: Vec<ResourceRecord>,
    ) -> Self {
        Self {
            source_uri,
            generation_sequence,
            committed_records: committed_records.into(),
        }
    }

    pub fn source_uri(&self) -> &AssetUri {
        &self.source_uri
    }

    pub fn generation_sequence(&self) -> u64 {
        self.generation_sequence
    }

    pub fn committed_records(&self) -> &[ResourceRecord] {
        self.committed_records.as_ref()
    }
}
