/// 根据分发清单选出的 crate 名生成当前平台产物名，供候选路径探测。
/// crate 名应由调用方验证，本函数只应用平台动态库命名约定。
pub(super) fn dynamic_library_file_name(crate_name: &str) -> String {
    #[cfg(target_os = "windows")]
    {
        exact_dynamic_library_name("", crate_name, ".dll")
    }
    #[cfg(target_os = "macos")]
    {
        exact_dynamic_library_name("lib", crate_name, ".dylib")
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        exact_dynamic_library_name("lib", crate_name, ".so")
    }
}

fn exact_dynamic_library_name(prefix: &str, crate_name: &str, suffix: &str) -> String {
    let capacity = prefix.len() + crate_name.len() + suffix.len();
    let mut name = String::with_capacity(capacity);
    name.push_str(prefix);
    name.push_str(crate_name);
    name.push_str(suffix);
    name
}

#[cfg(test)]
#[path = "tests/dynamic_library_name.rs"]
mod tests;
