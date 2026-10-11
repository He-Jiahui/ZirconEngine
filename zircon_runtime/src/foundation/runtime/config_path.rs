use std::path::PathBuf;

#[cfg(test)]
use std::cell::RefCell;
#[cfg(test)]
use std::marker::PhantomData;
#[cfg(test)]
use std::rc::Rc;
#[cfg(test)]
use std::sync::atomic::{AtomicU64, Ordering};
#[cfg(test)]
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(not(test))]
pub(super) fn config_file_path() -> PathBuf {
    // 默认选择器保留环境变量给出的原始路径，包括相对路径。
    // 宿主显式绑定要求绝对路径，由 new_with_file_path 单独验证。
    if let Some(path) = std::env::var_os("ZIRCON_CONFIG_PATH") {
        return PathBuf::from(path);
    }

    if cfg!(target_os = "windows") {
        if let Some(base) = std::env::var_os("LOCALAPPDATA").or_else(|| std::env::var_os("APPDATA"))
        {
            return PathBuf::from(base).join("ZirconEngine").join("config.json");
        }
    } else if let Some(base) = std::env::var_os("XDG_CONFIG_HOME") {
        return PathBuf::from(base).join("ZirconEngine").join("config.json");
    } else if let Some(home) = std::env::var_os("HOME") {
        return PathBuf::from(home)
            .join(".config")
            .join("ZirconEngine")
            .join("config.json");
    }

    PathBuf::from(".zircon-config.json")
}

#[cfg(test)]
pub(super) fn config_file_path() -> PathBuf {
    TEST_CONFIG_PATH_OVERRIDE
        .with(|path| path.borrow().clone())
        .unwrap_or_else(unique_test_config_file_path)
}

#[cfg(test)]
thread_local! {
    static TEST_CONFIG_PATH_OVERRIDE: RefCell<Option<PathBuf>> = const { RefCell::new(None) };
}

/// 在当前测试线程暂时覆盖新 manager 的默认配置路径。
///
/// # 使用时机与生命周期
///
/// 由 [override_config_file_path_for_test] 创建，并在需要该路径的构造调用期间持有。
/// 已有 worker 捕获的路径保持不变；Drop 恢复进入覆盖前的值，支持同线程嵌套。
/// Rc 标记使 guard 不能跨线程移动，确保还原操作发生在设置覆盖的线程。
#[cfg(test)]
pub(in crate::foundation) struct TestConfigPathOverride {
    previous: Option<PathBuf>,
    _not_send: PhantomData<Rc<()>>,
}

#[cfg(test)]
pub(in crate::foundation) fn override_config_file_path_for_test(
    path: PathBuf,
) -> TestConfigPathOverride {
    let previous = TEST_CONFIG_PATH_OVERRIDE.with(|current| current.replace(Some(path)));
    TestConfigPathOverride {
        previous,
        _not_send: PhantomData,
    }
}

#[cfg(test)]
impl Drop for TestConfigPathOverride {
    fn drop(&mut self) {
        let previous = self.previous.take();
        TEST_CONFIG_PATH_OVERRIDE.with(|current| {
            current.replace(previous);
        });
    }
}

#[cfg(test)]
fn unique_test_config_file_path() -> PathBuf {
    static NEXT_PATH_ID: AtomicU64 = AtomicU64::new(0);

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let path_id = NEXT_PATH_ID.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "zircon_config_test_{}_{}_{}.json",
        std::process::id(),
        timestamp,
        path_id
    ))
}
