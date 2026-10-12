mod commit_fence;
mod state;
mod worker;
mod writer;

use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use serde_json::Value;

use crate::core::framework::foundation::{
    ConfigManager, ConfigManagerError, ConfigPersistenceReport,
};
use crate::core::{CoreError, CoreHandle, CoreWeak};

use super::config_path::config_file_path;
use commit_fence::ConfigCommitFence;
use worker::ConfigPersistenceWorker;
use writer::{AtomicConfigFileWriter, ConfigFileWriter};

const DEFAULT_PERSISTENCE_DEBOUNCE: Duration = Duration::from_millis(25);
const DEFAULT_SHUTDOWN_FLUSH_TIMEOUT: Duration = Duration::from_secs(2);

/// 将 Core 配置存储与同一路径的异步持久化 worker 连接起来的默认服务。
///
/// # 使用时机
///
/// 构造时先注册路径 epoch、恢复并加载磁盘 JSON，再启动后台 worker。
/// 选择默认文件使用 [Self::new]；宿主预先绑定文件使用 [Self::new_with_file_path]。
///
/// # 所有权与生命周期
///
/// clone 共享路径和 worker。Core 回引用保持为弱句柄，避免服务注册表与运行时形成强引用环。
/// 最后一个 worker 所有者释放时执行关闭并 join 线程；关闭预算不是整个析构的耗时上限。
///
/// # 参考
///
/// 调用契约见 [ConfigManager]，磁盘确认见 [ConfigManager::flush]。
#[derive(Clone)]
pub struct DefaultConfigManager {
    // The registry owns this service, so its runtime back-reference must not complete an Arc cycle.
    core: CoreWeak,
    path: Arc<PathBuf>,
    persistence: Arc<ConfigPersistenceWorker>,
}

impl DefaultConfigManager {
    /// 使用默认路径选择器构造配置持久化服务。
    ///
    /// # 参数
    ///
    /// - `core`：本次服务所属的运行时；构造期间使用强句柄，服务内部只保留弱回引用。
    ///
    /// # 返回与错误
    ///
    /// 成功返回已加载现有配置并启动 worker 的服务。
    /// 路径 epoch 注册、备份恢复、文件读取、JSON 解析或线程创建失败会终止构造。
    ///
    /// # 约束与参考
    ///
    /// 默认选择器可保留相对路径。需要绝对的宿主文件绑定时使用 [Self::new_with_file_path]。
    pub fn new(core: &CoreHandle) -> Result<Self, CoreError> {
        Self::new_with_options(
            core,
            config_file_path(),
            Arc::new(AtomicConfigFileWriter),
            DEFAULT_PERSISTENCE_DEBOUNCE,
            DEFAULT_SHUTDOWN_FLUSH_TIMEOUT,
        )
    }

    /// Constructs the normal persistence manager with immutable host-owned file authority.
    /// Explicit paths must be absolute; the default constructor retains its existing selector.
    ///
    /// # 参数
    ///
    /// - `core`：服务所属运行时。
    /// - `path`：由宿主给出的非空绝对文件路径；构造后由服务和 worker 共享拥有。
    ///
    /// # 返回与生命周期
    ///
    /// 成功表示路径已验证、现有配置已加载且 worker 已启动；不表示文件此时一定存在。
    /// 构造错误与 [Self::new] 相同；空或相对路径会在注册和文件 I/O 前返回错误。
    /// 后续环境变量变化不会改写该服务捕获的路径。
    pub fn new_with_file_path(core: &CoreHandle, path: PathBuf) -> Result<Self, CoreError> {
        Self::validate_file_path(&path)?;
        Self::new_with_options(
            core,
            path,
            Arc::new(AtomicConfigFileWriter),
            DEFAULT_PERSISTENCE_DEBOUNCE,
            DEFAULT_SHUTDOWN_FLUSH_TIMEOUT,
        )
    }

    pub(in crate::foundation) fn validate_file_path(path: &PathBuf) -> Result<(), CoreError> {
        if path.as_os_str().is_empty() || !path.is_absolute() {
            return Err(config_error(
                path,
                "explicit config persistence file path must be nonempty and absolute".to_owned(),
            ));
        }
        Ok(())
    }

    /// 按指定 writer 和时间策略构造服务，供默认入口与可控测试共用。
    ///
    /// # 参数
    ///
    /// - `core`、`path`：配置所属运行时与本次固定的实际 I/O 路径。
    /// - `writer`：整份 JSON 快照的发布实现，须遵守 [ConfigFileWriter] 的 fence 契约。
    /// - `debounce`：合并普通脏请求的等待时间。
    /// - `shutdown_timeout`：关闭时等待 worker 终态的预算，不是 join 的截止时间。
    ///
    /// # 顺序与返回
    ///
    /// 先注册 [ConfigCommitFence]，再恢复及加载磁盘内容，最后启动 worker。
    /// 任一步失败即返回错误；此内部入口不代替显式路径入口的绝对路径检查。
    /// 成功返回的 clone 共享同一 worker，释放语义见 [DefaultConfigManager]。
    pub(super) fn new_with_options(
        core: &CoreHandle,
        path: PathBuf,
        writer: Arc<dyn ConfigFileWriter>,
        debounce: Duration,
        shutdown_timeout: Duration,
    ) -> Result<Self, CoreError> {
        let commit_fence = ConfigCommitFence::register(&path)
            .map_err(|error| config_error(&path, error.to_string()))?;
        recover_and_load_from_disk(core, &path, &commit_fence)?;
        let path = Arc::new(path);
        let persistence = ConfigPersistenceWorker::start(
            Arc::clone(&path),
            core.config_snapshot_source(),
            writer,
            debounce,
            shutdown_timeout,
            commit_fence,
        )?;
        Ok(Self {
            core: core.downgrade(),
            path,
            persistence,
        })
    }

    #[cfg(test)]
    pub(super) fn install_publication_test_hooks(
        &self,
        after_publication: impl FnOnce() + Send + 'static,
        on_contention: impl FnOnce() + Send + 'static,
    ) {
        self.persistence
            .install_publication_test_hooks(after_publication, on_contention);
    }

    fn runtime_core(&self) -> Result<CoreHandle, ConfigManagerError> {
        self.core
            .upgrade()
            .ok_or(ConfigManagerError::RuntimeUnavailable)
    }
}

impl fmt::Debug for DefaultConfigManager {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DefaultConfigManager")
            .field("path", &self.path)
            .field("persistence", &self.persistence)
            .finish()
    }
}

impl ConfigManager for DefaultConfigManager {
    fn set_value(&self, key: &str, value: Value) -> Result<(), ConfigManagerError> {
        let core = self.runtime_core()?;
        self.persistence
            .publish_and_request(|| core.store_config_value_if_changed(key, value));
        Ok(())
    }

    fn get_value(&self, key: &str) -> Option<Value> {
        self.core
            .upgrade()
            .and_then(|core| core.load_config_value(key))
    }

    fn flush(&self, timeout: Duration) -> Result<(), ConfigManagerError> {
        self.persistence.flush(timeout)
    }

    fn persistence_report(&self) -> ConfigPersistenceReport {
        self.persistence.report()
    }
}

fn recover_and_load_from_disk(
    core: &CoreHandle,
    path: &PathBuf,
    commit_fence: &ConfigCommitFence,
) -> Result<(), CoreError> {
    let json = commit_fence
        .commit(|| {
            crate::core::resource::io::recover_missing_target_from_backup(path)?;
            match fs::read_to_string(path) {
                Ok(json) => Ok(Some(json)),
                Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
                Err(error) => Err(error),
            }
        })
        .map_err(|error| config_error(path, error.to_string()))?;
    let Some(json) = json else {
        return Ok(());
    };
    // 先把整份 JSON 解析成对象，再逐键并入 Core；磁盘同名键覆盖已有值，缺键不会清空旧值。
    // 宿主 overlay 未在此分层；更高优先级的启动值由上层在加载后重新应用。
    let values = serde_json::from_str::<HashMap<String, Value>>(&json)
        .map_err(|error| config_error(path, error.to_string()))?;
    for (key, value) in values {
        core.store_config_value(key, value);
    }
    Ok(())
}

fn config_error(path: &PathBuf, message: String) -> CoreError {
    CoreError::ConfigParse(path.to_string_lossy().into_owned(), message)
}

#[cfg(test)]
pub(super) use commit_fence::ConfigCommitFence as ConfigCommitFenceForTest;
#[cfg(test)]
pub(super) use writer::ConfigFileWriter as ConfigFileWriterForTest;
