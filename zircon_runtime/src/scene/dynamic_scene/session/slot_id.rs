use super::RuntimeSessionArchiveError;

// 捕获、复制、导入与改名在创建新身份时调用；仅移除首尾空白并拒绝空值，不把 ID 解释为文件路径。
pub(super) fn normalize_slot_id(mut slot_id: String) -> Result<String, RuntimeSessionArchiveError> {
    trim_slot_id_in_place(&mut slot_id);
    validate_slot_id(&slot_id)?;
    Ok(slot_id)
}

// 复用调用者已拥有的缓冲区，避免批量导入为修剪再分配；字节边界来自字符串的 Unicode 空白判断。
fn trim_slot_id_in_place(slot_id: &mut String) {
    let trimmed_end = slot_id.trim_end().len();
    slot_id.truncate(trimmed_end);

    let trimmed_start = slot_id.len() - slot_id.trim_start().len();
    if trimmed_start != 0 {
        slot_id.drain(..trimmed_start);
    }
}

// 已构造或反序列化的槽位必须保留原身份进行校验；静默修剪会把不同输入折叠成同一索引键。
pub(super) fn validate_canonical_slot_id(slot_id: &str) -> Result<(), RuntimeSessionArchiveError> {
    validate_slot_id(slot_id)?;
    let canonical = slot_id.trim();
    if canonical != slot_id {
        return Err(RuntimeSessionArchiveError::NonCanonicalSlotId {
            slot_id: slot_id.to_string(),
            canonical: canonical.to_string(),
        });
    }
    Ok(())
}

// 槽位 ID 是档案内的不透明身份；此层仅要求存在非空白内容，唯一性由目标档案检查。
fn validate_slot_id(slot_id: &str) -> Result<(), RuntimeSessionArchiveError> {
    if slot_id.trim().is_empty() {
        return Err(RuntimeSessionArchiveError::EmptySlotId);
    }
    Ok(())
}

#[cfg(test)]
#[path = "slot_id/tests/in_place_tests.rs"]
mod in_place_tests;
