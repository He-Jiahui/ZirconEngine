// 标签去掉首尾空白后按字符串顺序排序去重；保留大小写区别，以便存储和查询共用确定的标签集合。
pub(super) fn normalize_metadata_tags(tags: &mut Vec<String>) {
    for tag in tags.iter_mut() {
        trim_tag_in_place(tag);
    }
    tags.retain(|tag| !tag.is_empty());
    tags.sort();
    tags.dedup();
}

fn trim_tag_in_place(tag: &mut String) {
    let trimmed_end = tag.trim_end().len();
    tag.truncate(trimmed_end);

    // 长度来自字符串裁剪后的 UTF-8 边界，前缀移除不会截断字符；沿用原有字符串缓冲区。
    let trimmed_start = tag.len() - tag.trim_start().len();
    if trimmed_start != 0 {
        tag.drain(..trimmed_start);
    }
}

#[cfg(test)]
#[path = "tags/tests/in_place_tests.rs"]
mod in_place_tests;
