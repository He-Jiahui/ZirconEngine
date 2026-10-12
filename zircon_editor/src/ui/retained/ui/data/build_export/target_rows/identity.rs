//! 目标键用于原生行和节点身份；重复平台附加配置名以区分宿主节点，执行动作的 ID 另由导出预设名生成。
pub(super) fn build_export_target_id(
    platform_id: &str,
    profile_name: &str,
    duplicate_platform: bool,
) -> String {
    if duplicate_platform {
        let mut target = String::with_capacity(platform_id.len() + 1 + profile_name.len());
        target.push_str(platform_id);
        target.push('.');
        push_build_export_key(&mut target, profile_name);
        target
    } else {
        platform_id.to_string()
    }
}

pub(super) fn build_export_key(value: &str) -> String {
    let mut key = String::with_capacity(value.len());
    push_build_export_key(&mut key, value);
    key
}

fn push_build_export_key(target: &mut String, value: &str) {
    let segment_start = target.len();
    let mut started = false;
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            target.push(ch.to_ascii_lowercase());
            started = true;
        } else if started {
            target.push('_');
        }
    }
    while target.len() > segment_start && target.ends_with('_') {
        target.pop();
    }
    if target.len() == segment_start {
        target.push_str("target");
    }
}

#[cfg(test)]
#[path = "tests/identity.rs"]
mod tests;
