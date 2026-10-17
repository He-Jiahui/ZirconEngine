use std::collections::{btree_map::Entry, BTreeMap};
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::{hub_recent_project_path_key, HubRecentProjectV1, HubRecentProjectsError};
use crate::hub_protocol::HubProtocolVersionV1;

pub const HUB_RECENT_PROJECT_LIMIT_V1: usize = 8;
pub const HUB_RECENT_PROJECT_TOMBSTONE_LIMIT_V1: usize = 64;

/// Durable evidence that an entry was deliberately removed from the rebuildable recent projection.
///
/// Tombstones are keyed by the same normalized display-path key as v1 entries. Stable project and
/// filesystem identity remain preflight-owned; this projection never promotes a display path into
/// a project identity authority.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// 记录最近项目被显式移除的规范路径键和删除逻辑时刻。
///
/// `remove` 创建墓碑；重新 `record` 同一键时将其移除，墓碑时间也参与后续逻辑时钟。
/// 从外部解码后须通过 [`HubRecentProjectsV1::validate`] 检查整个投影的约束。
pub struct HubRecentProjectTombstoneV1 {
    #[serde(deserialize_with = "deserialize_tombstone_path_key")]
    path_key: String,
    deleted_logical_unix_ms: u64,
}

// 外部墓碑必须沿用项目条目的路径键规则，防止大小写或分隔符变体绕过互斥检查。
fn deserialize_tombstone_path_key<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let path_key = String::deserialize(deserializer)?;
    Ok(hub_recent_project_path_key(path_key))
}

impl HubRecentProjectTombstoneV1 {
    fn new(path_key: String, deleted_logical_unix_ms: u64) -> Self {
        Self {
            path_key,
            deleted_logical_unix_ms,
        }
    }

    pub fn path_key(&self) -> &str {
        &self.path_key
    }

    pub const fn deleted_logical_unix_ms(&self) -> u64 {
        self.deleted_logical_unix_ms
    }
}

/// Canonical on-disk document shared between the Hub and Editor.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Hub 与 Editor 共享的有界最近项目文件投影。
///
/// 路径键只用于展示记录去重；项目身份和打开权限由宿主预检。字段公开且 Serde 派生
/// 不会调用 [`Self::validate`]，直接构造或解码后应先校验；持久化交给
/// [`crate::hub_protocol::HubRecentProjectsStore`] 在写锁内完成读改写。
pub struct HubRecentProjectsV1 {
    pub protocol_version: HubProtocolVersionV1,
    pub revision: u64,
    pub projects: Vec<HubRecentProjectV1>,
    pub tombstones: Vec<HubRecentProjectTombstoneV1>,
}

impl HubRecentProjectsV1 {
    /// 从项目条目创建规范排序的初始投影。
    ///
    /// # 参数
    /// - `projects`：将被按共享路径键去重、排序并截断的记录；条目有效性由调用方保证。
    ///
    /// # 返回值与边界
    /// 空投影的 revision 为 0，非空为 1，墓碑为空。此构造只合并条目，不执行
    /// [`Self::validate`] 或文件读写；已有文档的持续更新应使用文件事务。
    ///
    /// # 参考
    /// [`merge_hub_recent_projects`]；[`crate::hub_protocol::HubRecentProjectsStore::update`]。
    pub fn new(projects: impl IntoIterator<Item = HubRecentProjectV1>) -> Self {
        let projects = merge_hub_recent_projects(projects, []);
        Self {
            protocol_version: HubProtocolVersionV1,
            revision: u64::from(!projects.is_empty()),
            projects,
            tombstones: Vec::new(),
        }
    }

    pub const fn revision(&self) -> u64 {
        self.revision
    }

    /// 按逻辑时钟推进打开时间，再以规范化路径合并项目并清除同路径墓碑；revision 随此次变更递增。
    ///
    /// # 参数
    /// - `project`：已校验的展示记录；其打开时间是提议值，会推进到所有项目和墓碑时间之后。
    ///
    /// # 返回值与约束
    /// 成功时按路径键合并、保留最近的有限条目，并清除该规范键的墓碑。逻辑时钟或
    /// revision 耗尽时返回错误，赋值尚未提交到 `self`；本方法不执行整个投影的校验。
    ///
    /// # 参考
    /// [`Self::validate`]；文件更新使用 [`crate::hub_protocol::HubRecentProjectsStore::update`]。
    pub fn record(
        &mut self,
        mut project: HubRecentProjectV1,
    ) -> Result<(), HubRecentProjectsError> {
        project.last_opened_unix_ms = self.next_logical_timestamp(project.last_opened_unix_ms)?;
        let path_key = hub_recent_project_path_key(&project.path);
        let projects = merge_hub_recent_projects(self.projects.iter().cloned(), [project]);
        self.bump_revision()?;
        self.projects = projects;
        self.tombstones
            .retain(|tombstone| tombstone.path_key != path_key);
        Ok(())
    }

    /// 从当前投影移除路径并记录墓碑，使较旧的合并输入不能把用户已删除的项目重新带回。
    ///
    /// # 参数
    /// - `path`：按共享词法规则生成键的展示路径，不要求文件仍然存在。
    ///
    /// # 返回值与约束
    /// 项目已不存在且同键墓碑已存在时成功返回且不推进 revision；其余情况生成删除墓碑。
    /// 逻辑时钟或 revision 耗尽均在修改集合前返回错误，内存投影保持不变。
    /// 文件事务的失败副本不会写回。
    ///
    /// # 参考
    /// [`crate::hub_protocol::hub_recent_project_path_key`]；[`Self::record`]。
    pub fn remove(&mut self, path: impl AsRef<Path>) -> Result<(), HubRecentProjectsError> {
        let path_key = hub_recent_project_path_key(path);
        let project_exists = self
            .projects
            .iter()
            .any(|project| hub_recent_project_path_key(&project.path) == path_key);
        if !project_exists
            && self
                .tombstones
                .iter()
                .any(|tombstone| tombstone.path_key == path_key)
        {
            return Ok(());
        }
        let deleted_logical_unix_ms = self.next_tombstone_timestamp()?;
        // 所有可恢复错误均在修改集合前返回。
        self.bump_revision()?;
        self.projects
            .retain(|project| hub_recent_project_path_key(&project.path) != path_key);
        self.tombstones
            .retain(|tombstone| tombstone.path_key != path_key);
        self.tombstones.push(HubRecentProjectTombstoneV1::new(
            path_key,
            deleted_logical_unix_ms,
        ));
        self.canonicalize_tombstones();
        Ok(())
    }

    /// 文件解码和写回前都用此检查拒绝超限、重复或未按规范排序的投影与墓碑。
    ///
    /// # 返回值
    /// 成功表示容量、项目路径、规范键唯一性、项目与墓碑互斥和排序检查通过；
    /// 按源码检查顺序返回首个错误。时间只参与排序，不额外校验与 revision 的关系。
    ///
    /// 本方法不访问文件系统；协议版本由标记类型的 Serde 边界校验，墓碑路径键在解码时归一化。
    ///
    /// # 参考
    /// [`HubRecentProjectV1`]；[`crate::hub_protocol::HubRecentProjectsStore::load_projection`]。
    pub fn validate(&self) -> Result<(), HubRecentProjectsError> {
        if self.projects.len() > HUB_RECENT_PROJECT_LIMIT_V1 {
            return Err(HubRecentProjectsError::TooManyEntries {
                limit: HUB_RECENT_PROJECT_LIMIT_V1,
            });
        }
        if self.tombstones.len() > HUB_RECENT_PROJECT_TOMBSTONE_LIMIT_V1 {
            return Err(HubRecentProjectsError::TooManyTombstones {
                limit: HUB_RECENT_PROJECT_TOMBSTONE_LIMIT_V1,
            });
        }

        let mut keys = BTreeMap::new();
        let mut previous_project = None;
        let mut canonical_order = true;
        for project in &self.projects {
            project.validate()?;
            let key = hub_recent_project_path_key(&project.path);
            match keys.entry(key) {
                Entry::Vacant(entry) => {
                    entry.insert(());
                }
                Entry::Occupied(entry) => {
                    return Err(HubRecentProjectsError::DuplicateProjectPath {
                        path_key: entry.key().clone(),
                    });
                }
            }
            if canonical_order
                && previous_project
                    .is_some_and(|previous| !is_canonical_successor(previous, project))
            {
                canonical_order = false;
            }
            previous_project = Some(project);
        }

        if !canonical_order {
            return Err(HubRecentProjectsError::NonCanonicalOrder);
        }
        let mut tombstone_keys = BTreeMap::new();
        let mut previous_tombstone = None;
        for tombstone in &self.tombstones {
            if tombstone.path_key.is_empty() {
                return Err(HubRecentProjectsError::EmptyTombstonePathKey);
            }
            if keys.contains_key(&tombstone.path_key) {
                return Err(HubRecentProjectsError::TombstoneOverlapsProject {
                    path_key: tombstone.path_key.clone(),
                });
            }
            match tombstone_keys.entry(tombstone.path_key.clone()) {
                Entry::Vacant(entry) => {
                    entry.insert(());
                }
                Entry::Occupied(entry) => {
                    return Err(HubRecentProjectsError::DuplicateTombstonePathKey {
                        path_key: entry.key().clone(),
                    });
                }
            }
            if previous_tombstone.is_some_and(|previous: &HubRecentProjectTombstoneV1| {
                !is_canonical_tombstone_successor(previous, tombstone)
            }) {
                return Err(HubRecentProjectsError::NonCanonicalTombstoneOrder);
            }
            previous_tombstone = Some(tombstone);
        }
        Ok(())
    }

    fn bump_revision(&mut self) -> Result<(), HubRecentProjectsError> {
        self.revision = self
            .revision
            .checked_add(1)
            .ok_or(HubRecentProjectsError::RevisionExhausted)?;
        Ok(())
    }

    fn next_logical_timestamp(&self, proposed_unix_ms: u64) -> Result<u64, HubRecentProjectsError> {
        match self.latest_logical_timestamp() {
            Some(current) => current
                .checked_add(1)
                .map(|next| proposed_unix_ms.max(next))
                .ok_or(HubRecentProjectsError::LogicalClockExhausted),
            None => Ok(proposed_unix_ms),
        }
    }

    fn next_tombstone_timestamp(&self) -> Result<u64, HubRecentProjectsError> {
        self.latest_logical_timestamp().map_or(Ok(1), |current| {
            current
                .checked_add(1)
                .ok_or(HubRecentProjectsError::LogicalClockExhausted)
        })
    }

    fn latest_logical_timestamp(&self) -> Option<u64> {
        self.projects
            .iter()
            .map(|project| project.last_opened_unix_ms)
            .chain(
                self.tombstones
                    .iter()
                    .map(|tombstone| tombstone.deleted_logical_unix_ms),
            )
            .max()
    }

    fn canonicalize_tombstones(&mut self) {
        self.tombstones.sort_by(|left, right| {
            right
                .deleted_logical_unix_ms
                .cmp(&left.deleted_logical_unix_ms)
                .then_with(|| left.path_key.cmp(&right.path_key))
        });
        self.tombstones
            .truncate(HUB_RECENT_PROJECT_TOMBSTONE_LIMIT_V1);
    }
}

fn is_canonical_successor(previous: &HubRecentProjectV1, current: &HubRecentProjectV1) -> bool {
    previous.last_opened_unix_ms > current.last_opened_unix_ms
        || (previous.last_opened_unix_ms == current.last_opened_unix_ms
            && previous.path <= current.path)
}

fn is_canonical_tombstone_successor(
    previous: &HubRecentProjectTombstoneV1,
    current: &HubRecentProjectTombstoneV1,
) -> bool {
    previous.deleted_logical_unix_ms > current.deleted_logical_unix_ms
        || (previous.deleted_logical_unix_ms == current.deleted_logical_unix_ms
            && previous.path_key <= current.path_key)
}

impl Default for HubRecentProjectsV1 {
    fn default() -> Self {
        Self {
            protocol_version: HubProtocolVersionV1,
            revision: 0,
            projects: Vec::new(),
            tombstones: Vec::new(),
        }
    }
}

/// Merges two registry snapshots without retaining a host-specific authority field.
///
/// The newest timestamp wins; identical timestamps use manifest name as a deterministic tie
/// break. The resulting collection is path-deduplicated, ordered, and bounded.
///
/// # 参数
/// - `left`、`right`：移交所有权的项目记录迭代器；调用方须保证条目满足自身契约。
///
/// # 返回值与边界
/// 按规范路径键去重，保留较新时间；时间相同则选名称字典序较小者。结果按时间降序、
/// 原始路径升序排列并截为最近项目上限。此入口只接收项目条目，不接收或应用墓碑，
/// 也不验证文件系统身份。
///
/// # 参考
/// [`crate::hub_protocol::hub_recent_project_path_key`]；[`HUB_RECENT_PROJECT_LIMIT_V1`]。
pub fn merge_hub_recent_projects<I, J>(left: I, right: J) -> Vec<HubRecentProjectV1>
where
    I: IntoIterator<Item = HubRecentProjectV1>,
    J: IntoIterator<Item = HubRecentProjectV1>,
{
    let mut projects = BTreeMap::<String, HubRecentProjectV1>::new();
    for project in left.into_iter().chain(right) {
        let key = hub_recent_project_path_key(&project.path);
        match projects.get(&key) {
            Some(existing) if existing.last_opened_unix_ms > project.last_opened_unix_ms => {}
            Some(existing)
                if existing.last_opened_unix_ms == project.last_opened_unix_ms
                    && existing.summary.name <= project.summary.name => {}
            _ => {
                projects.insert(key, project);
            }
        }
    }
    let mut merged = projects.into_values().collect::<Vec<_>>();
    merged.sort_by(|left, right| {
        right
            .last_opened_unix_ms
            .cmp(&left.last_opened_unix_ms)
            .then_with(|| left.path.cmp(&right.path))
    });
    merged.truncate(HUB_RECENT_PROJECT_LIMIT_V1);
    merged
}

#[cfg(test)]
#[path = "tests/registry_perf.rs"]
mod performance_tests;
