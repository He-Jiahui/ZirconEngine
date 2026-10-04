use crate::{ResourceRecord, ResourceRegistryError, ResourceResult};

// 版本代表记录中的内容身份；载荷地址或诊断变化不会自动形成新内容版本，调用端需更新来源、导入配置等身份字段。
// TODO: [CR-RESOURCE-AUDIT-0002] 确认各 Ready 发布入口都能把载荷内容变化反映到元数据身份；当前同元数据替换载荷不会增版或发出记录变化事件，需补齐直接注册与导入路径的契约验证。
pub(super) fn next_ready_revision(
    previous: &ResourceRecord,
    next: &ResourceRecord,
) -> ResourceResult<u64> {
    if previous.revision == 0 {
        Ok(1)
    } else if ready_record_changed(previous, next) {
        previous
            .revision
            .checked_add(1)
            .ok_or_else(|| ResourceRegistryError::RevisionExhausted {
                id: previous.id.to_string(),
                current_revision: previous.revision,
            })
    } else {
        Ok(previous.revision)
    }
}

fn ready_record_changed(previous: &ResourceRecord, next: &ResourceRecord) -> bool {
    previous.kind != next.kind
        || previous.primary_locator != next.primary_locator
        || previous.artifact_locator != next.artifact_locator
        || previous.source_hash != next.source_hash
        || previous.importer_id != next.importer_id
        || previous.importer_version != next.importer_version
        || previous.config_hash != next.config_hash
        || previous.dependency_ids != next.dependency_ids
}
