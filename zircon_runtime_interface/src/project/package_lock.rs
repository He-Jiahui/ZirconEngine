use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

use super::{ProjectGuid, ProjectManifestDigest};

/// 锁主体的 V1 协议版本；[ProjectPackageLock::validate] 要求 `schema_version` 等于此值。
/// 外层状态另用 [PROJECT_PACKAGE_LOCK_STATE_SCHEMA_VERSION_V1]。
pub const PROJECT_PACKAGE_LOCK_SCHEMA_VERSION_V1: u32 = 1;
/// 包锁状态封装的 V1 协议版本，适用于 `Present` 和 `Unavailable` 两个分支。
/// [ProjectPackageLockState::validate] 独立校验此版本和内部锁主体。
pub const PROJECT_PACKAGE_LOCK_STATE_SCHEMA_VERSION_V1: u32 = 1;
/// 一个目标快照最多包含 512 个插件条目，由 [ProjectPackageLock::validate] 检查。
/// 满足条目数限制后，编码仍须满足 [MAX_PROJECT_PACKAGE_LOCK_BYTES]。
pub const MAX_PROJECT_PACKAGE_LOCK_ENTRIES: usize = 512;
/// 每个包条目最多包含 64 项能力；不是整个锁的能力总数上限。
/// [ProjectPackageLock::validate] 将超限归为 [ProjectPackageLockError::InvalidIdentity]。
pub const MAX_PROJECT_PACKAGE_LOCK_CAPABILITIES: usize = 64;
/// 插件标识和单项能力文本的 UTF-8 字节上限，值为 128。
/// 具体字符规则见 [ProjectPackageLockEntry]，由 [ProjectPackageLock::validate] 检查。
pub const MAX_PROJECT_PACKAGE_LOCK_TEXT: usize = 128;
// Leave space for the existing 64 KiB helper response envelope and newline.
/// 单个锁主体 JSON 的字节预算，为 60 KiB；helper 外层字段和换行另占回复空间。
/// [ProjectPackageLock::canonical_bytes] 检查规范编码长度，
/// [ProjectPackageLock::decode_canonical] 还先检查原始输入长度。
pub const MAX_PROJECT_PACKAGE_LOCK_BYTES: usize = 60 * 1024;

/// 标记包选择所针对的客户端运行时或编辑器宿主。
///
/// 此值与平台共同组成 [ProjectPackageLockTarget]，参与共享依赖摘要。
/// JSON 使用 `client_runtime` 和 `editor_host`；该类型没有服务器运行模式分支。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectPackageLockRuntimeMode {
    ClientRuntime,
    EditorHost,
}

/// 标记包选择所针对的桌面平台，供生产端与消费端核对同一目标。
///
/// JSON 使用 `windows`、`linux` 和 `macos`。平台与运行模式组成
/// [ProjectPackageLockTarget]；此枚举不携带 CPU 架构信息。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectPackageLockPlatform {
    Windows,
    Linux,
    Macos,
}

/// 以运行模式和平台确定一次包选择快照的目标。
///
/// `runtime_mode` 选择客户端运行时或编辑器宿主，`platform` 选择桌面平台。
/// JSON 字段使用 camelCase，并拒绝未知字段。
///
/// 目标参与 [ProjectPackageLock::digest]。锁的通用校验没有期望目标参数，
/// 消费入口须将此值与当前请求的目标比较。
///
/// # 参考
///
/// [ProjectPackageLockRuntimeMode]、[ProjectPackageLockPlatform]。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProjectPackageLockTarget {
    pub runtime_mode: ProjectPackageLockRuntimeMode,
    pub platform: ProjectPackageLockPlatform,
}

/// 把包选择快照关联到一个本地项目及捕获时的清单字节。
///
/// `project_guid` 使用 [ProjectGuid] 的非 nil 身份；`manifest_digest` 使用
/// [ProjectManifestDigest]，其值标识清单原始字节的 BLAKE3 摘要。
///
/// 此对象随锁编码保留，供生产和消费入口核对项目及清单代际。
/// [ProjectPackageLock::digest] 排除整个 `project` 对象，允许本地身份不同的
/// 签出共享同一依赖指纹。锁的通用校验不会替消费入口比较期望身份。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProjectPackageLockProject {
    pub project_guid: ProjectGuid,
    pub manifest_digest: ProjectManifestDigest,
}

/// 记录一个插件在指定目标上的包版本、发布修订、工件身份和能力选择。
///
/// # 字段约束
///
/// - `plugin_id` 是最多 128 字节的点分标识；各段以小写 ASCII 字母开始，
///   后续只含小写字母、数字或下划线，不能以 `_` 结尾或包含 `__`。
/// - `package_id` 使用 UUID 的规范小写连字符表示。
/// - `version` 使用 [valid_package_version] 接受的三个 `u32` 十进制段。
/// - `release_revision` 是大于零的 `u64` 规范十进制表示。
/// - `artifact_digest` 是 64 个小写十六进制字符工件身份文本。
/// - `capabilities` 最多 64 项；每项非空、最多 128 个 UTF-8 字节，
///   没有首尾空白或控制字符，同一条目中不能重复。
///
/// # 使用边界
///
/// 这些是拥有字符串的值。构造和 Serde 解码不会自动排序或执行上述语义校验。
/// [ProjectPackageLock::validate] 还要求整个锁中插件标识唯一；
/// [ProjectPackageLock::canonical_bytes] 在副本中排序条目和能力。
/// 字段本身不能证明对应包仍存在或已被授予权限。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProjectPackageLockEntry {
    pub plugin_id: String,
    pub package_id: String,
    pub version: String,
    pub release_revision: String,
    pub artifact_digest: String,
    pub capabilities: Vec<String>,
}

/// Producer provenance captured from the authenticated native policy loader and
/// host package-store generation. The runtime producer obtains these values
/// from `LoadedNativePluginPolicy`; cloud consumers validate shape and digest
/// linkage only. This witness is not a server-verifiable authorization token,
/// so Hub must never grant authority from client-supplied fields alone.
///
/// 记录生产端捕获时的本机策略与安装代际见证。
///
/// # 字段来源
///
/// - `index_sha256`：已加载策略索引原始字节的 SHA-256。
/// - `policy_sha256`：索引指定并由加载器核对的策略文件 SHA-256。
/// - `build_set_id`：已加载策略携带的构建集合标识。
/// - `capability_digest`：策略允许能力排序、去重后，其 JSON 的 SHA-256。
/// - `provider_revision_digest`：相关安装选择的身份摘要与库存修订组成二元组，
///   将列表排序后计算其 JSON 的 SHA-256。
///
/// # 验证与生命周期
///
/// [ProjectPackageLock::validate] 仅要求这五个字符串均为 64 个小写十六进制字符。
/// 它们参与 [ProjectPackageLock::digest]，描述捕获时的来源与代际；
/// 这些值不会自动更新，也不能单凭客户端提供的字段证明服务端授权。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProjectPackageLockAuthority {
    pub index_sha256: String,
    pub policy_sha256: String,
    pub build_set_id: String,
    pub capability_digest: String,
    pub provider_revision_digest: String,
}

/// 面向指定运行模式和平台的包选择快照，供 Hub 与云清单核对依赖。
/// 项目身份与目标由消费入口另行核对，authority 保存生产端的来源见证。
///
/// # 结构与使用
///
/// `schema_version` 标记锁主体协议；`project` 保存本地捕获身份；
/// `target` 区分依赖目标；`authority` 保存来源见证；`entries` 保存插件选择。
/// 空 `entries` 是一个选择为空的快照，仍有目标和来源，并可产生真实摘要。
///
/// # 构造与验证边界
///
/// 公开字段允许直接构造和修改，Serde 解码只建立值及类型形状。
/// [ProjectPackageLock::validate] 检查条目和来源格式；
/// [ProjectPackageLock::canonical_bytes] 还检查编码容量并在副本中排序；
/// [ProjectPackageLock::require_canonical_wire] 要求当前对象已经按规范排序。
/// 消费入口另行核对预期项目身份和目标，并从可信捕获路径取得该快照。
///
/// 这是同步使用的拥有数据值，不持有捕获文件或策略加载器句柄。
/// 修改字段后，应重新执行所需校验并用 [ProjectPackageLock::digest] 重算摘要。
///
/// # 参考
///
/// [ProjectPackageLockProject]、[ProjectPackageLockTarget]、
/// [ProjectPackageLockAuthority]、[ProjectPackageLockEntry]、[ProjectPackageLockState]。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProjectPackageLock {
    pub schema_version: u32,
    pub project: ProjectPackageLockProject,
    pub target: ProjectPackageLockTarget,
    pub authority: ProjectPackageLockAuthority,
    pub entries: Vec<ProjectPackageLockEntry>,
}

/// 为 [ProjectPackageLockState::Unavailable] 携带提供方报告的不可用分类。
///
/// JSON 分类值为 `provider_missing`、`incompatible`、`generation_changed`
/// 和 `invalid`。状态校验接受这些类型分支，不据此生成锁或证明依赖可用。
/// 要求有锁的消费入口应检查状态分支；此枚举不规定重试、恢复或错误映射。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectPackageLockUnavailableReason {
    ProviderMissing,
    Incompatible,
    GenerationChanged,
    Invalid,
}

/// 有锁的 Present 与不可用原因分开表示；空选择也可以是合法 Present。
/// validate 接受形状合法的 Unavailable，要求有锁时由消费入口检查分支。
///
/// # 状态与内容
///
/// - `Present` 拥有一个原始锁对象和其共享摘要 `digest`；外层
///   `schema_version` 使用状态协议版本，不替代内部锁的版本。
/// - `Unavailable` 仅携带状态协议版本和 [ProjectPackageLockUnavailableReason]，
///   没有可供消费的锁或共享摘要。
///
/// JSON 通过 `state` 字段区分 `present` 与 `unavailable`，版本字段名为
/// `schemaVersion`。所有内容均是同步持有的值，没有后续捕获任务。
///
/// # 构造与消费
///
/// [ProjectPackageLockState::present] 保留输入锁；它不会将输入对象排序。
/// 消费前使用 [ProjectPackageLockState::validate] 检查版本、顺序和摘要关联，
/// 再按业务需要要求 `Present`。项目身份、目标与授权仍由消费入口核对。
/// 修改内部锁后，必须重算状态摘要并重新校验。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case", deny_unknown_fields)]
pub enum ProjectPackageLockState {
    Present {
        #[serde(rename = "schemaVersion")]
        schema_version: u32,
        lock: Box<ProjectPackageLock>,
        digest: String,
    },
    Unavailable {
        #[serde(rename = "schemaVersion")]
        schema_version: u32,
        reason: ProjectPackageLockUnavailableReason,
    },
}

/// 描述包锁形状、编码或状态关联未满足契约的原因。
///
/// # 错误含义
///
/// - `UnsupportedSchema`：锁主体或外层状态协议版本不支持。
/// - `Capacity`：条目数、规范编码长度或原始解码输入长度超出相应上限。
/// - `InvalidText`：能力文本无效或同一条目内能力重复。
/// - `InvalidIdentity`：插件标识、包 UUID、发布修订、工件摘要不合法，
///   或单条目的能力项数超限。
/// - `DuplicatePlugin`：整个锁中插件标识重复。
/// - `Encoding`：JSON 编码、解码或类型形状转换失败。
/// - `NonCanonicalWire`：对象未按规范排序，或 `Present` 的摘要与锁不一致。
/// - `InvalidVersion`：包版本不满足 [valid_package_version] 的数值和语法域。
/// - `InvalidAuthority`：来源见证的任一字符串不满足摘要文本形状。
///
/// 这些分类不携带期望项目身份或目标，也不授予权限。校验与编码方法只读借用锁，
/// 失败不改写该锁；[ProjectPackageLockState::present] 则取得输入锁的所有权。
///
/// # 参考
///
/// [ProjectPackageLock::validate]、[ProjectPackageLock::decode_canonical]、
/// [ProjectPackageLockState::validate]。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProjectPackageLockError {
    UnsupportedSchema,
    Capacity,
    InvalidText,
    InvalidIdentity,
    DuplicatePlugin,
    Encoding,
    NonCanonicalWire,
    InvalidVersion,
    InvalidAuthority,
}

impl ProjectPackageLock {
    /// 验证形状后在副本中排序条目和能力，生成受容量限制的 JSON。
    /// 原锁顺序保持不变；要求原对象已规范排序时使用 require_canonical_wire。
    ///
    /// # 返回与错误
    ///
    /// 成功返回完整锁对象的紧凑 JSON 字节，长度不超过
    /// [MAX_PROJECT_PACKAGE_LOCK_BYTES]；本地 `project` 身份仍在编码中。
    /// 先传播 [ProjectPackageLock::validate] 的错误；JSON 编码失败返回
    /// [ProjectPackageLockError::Encoding]，编码超出预算返回
    /// [ProjectPackageLockError::Capacity]。
    ///
    /// # 调用约束
    ///
    /// 此方法只读借用锁，在克隆中按 `plugin_id`、`package_id`、
    /// `release_revision`、`artifact_digest` 的字符串字典序排序条目，
    /// 再按字符串字典序排序各条目的能力。修订排序不是数值排序。
    /// 成功与失败都不会修改原对象，也不核对消费入口的期望项目或目标。
    ///
    /// # 参考
    ///
    /// [ProjectPackageLock::require_canonical_wire]、[ProjectPackageLock::decode_canonical]、
    /// [ProjectPackageLock::digest]。
    pub fn canonical_bytes(&self) -> Result<Vec<u8>, ProjectPackageLockError> {
        self.validate()?;
        let mut normalized = self.clone();
        normalized.entries.sort_by(|left, right| {
            left.plugin_id
                .cmp(&right.plugin_id)
                .then(left.package_id.cmp(&right.package_id))
                .then(left.release_revision.cmp(&right.release_revision))
                .then(left.artifact_digest.cmp(&right.artifact_digest))
        });
        for entry in &mut normalized.entries {
            entry.capabilities.sort();
        }
        let bytes =
            serde_json::to_vec(&normalized).map_err(|_| ProjectPackageLockError::Encoding)?;
        if bytes.len() > MAX_PROJECT_PACKAGE_LOCK_BYTES {
            return Err(ProjectPackageLockError::Capacity);
        }
        Ok(bytes)
    }

    /// 对规范化后的版本、目标、authority 和包条目计算共享依赖指纹。
    /// 本地 project GUID 与 manifest 摘要保留在锁中供入口核对，并整体排除在此指纹之外。
    ///
    /// # 返回与错误
    ///
    /// 返回 64 个小写十六进制字符 SHA-256，用于比较共享依赖选择。
    /// 哈希输入是规范化后 `(schema_version, target, authority, entries)`
    /// 四元组的 JSON 字节，整个 `project` 对象均被排除。
    ///
    /// 先执行 [ProjectPackageLock::canonical_bytes]，因此即使摘要输入不含项目身份，
    /// 完整锁仍须满足该方法的形状与容量约束。中间 JSON 转换失败返回
    /// [ProjectPackageLockError::Encoding]，其余错误从规范编码过程传播。
    ///
    /// # 调用约束
    ///
    /// 借用和返回期间均不修改锁。相同摘要不证明本地项目身份、清单代际、
    /// 当前安装状态或服务端授权相同；消费入口必须单独核对这些事实。
    ///
    /// # 参考
    ///
    /// [ProjectPackageLockProject]、[ProjectPackageLockAuthority]、[ProjectPackageLockState]。
    pub fn digest(&self) -> Result<String, ProjectPackageLockError> {
        // Local checkout identity authenticates the witness, but is never part of
        // the shared dependency fingerprint: teammates have different project GUIDs.
        let normalized: Self = serde_json::from_slice(&self.canonical_bytes()?)
            .map_err(|_| ProjectPackageLockError::Encoding)?;
        let bytes = serde_json::to_vec(&(
            normalized.schema_version,
            normalized.target,
            normalized.authority,
            normalized.entries,
        ))
        .map_err(|_| ProjectPackageLockError::Encoding)?;
        Ok(format!("{:x}", Sha256::digest(bytes)))
    }

    /// 校验版本、包条目身份及唯一性、能力文本和 authority 摘要格式。
    /// 总编码容量由 canonical_bytes 检查，项目身份和目标由消费入口核对。
    ///
    /// # 返回与错误
    ///
    /// 成功表示主体版本、条目数、各条目格式、插件唯一性和来源见证形状有效。
    /// 按检查顺序返回首个错误：主体版本为 `UnsupportedSchema`，条目数为
    /// `Capacity`，包版本为 `InvalidVersion`，标识或能力项数为 `InvalidIdentity`，
    /// 能力文本或重复能力为 `InvalidText`，重复插件为 `DuplicatePlugin`，
    /// 来源字符串格式为 `InvalidAuthority`。详见 [ProjectPackageLockError]。
    ///
    /// # 调用约束
    ///
    /// 仅同步读取当前值，不排序、不写回，也不确认包或能力真实存在。
    /// 当前条目和能力可以无序；完整 JSON 字节预算由
    /// [ProjectPackageLock::canonical_bytes] 检查，对象顺序由
    /// [ProjectPackageLock::require_canonical_wire] 检查。
    /// 项目 GUID、清单摘要和目标与期望值的比较由消费入口承担。
    ///
    /// # 参考
    ///
    /// [ProjectPackageLockEntry]、[ProjectPackageLockAuthority]、[valid_package_version]。
    pub fn validate(&self) -> Result<(), ProjectPackageLockError> {
        if self.schema_version != PROJECT_PACKAGE_LOCK_SCHEMA_VERSION_V1 {
            return Err(ProjectPackageLockError::UnsupportedSchema);
        }
        if self.entries.len() > MAX_PROJECT_PACKAGE_LOCK_ENTRIES {
            return Err(ProjectPackageLockError::Capacity);
        }
        let mut plugins = BTreeSet::new();
        for entry in &self.entries {
            if !valid_package_version(&entry.version) {
                return Err(ProjectPackageLockError::InvalidVersion);
            }
            if !valid_plugin_id(&entry.plugin_id)
                || !canonical_uuid(&entry.package_id)
                || !canonical_revision(&entry.release_revision)
                || !is_digest(&entry.artifact_digest)
                || entry.capabilities.len() > MAX_PROJECT_PACKAGE_LOCK_CAPABILITIES
            {
                return Err(ProjectPackageLockError::InvalidIdentity);
            }
            let mut capabilities = BTreeSet::new();
            for capability in &entry.capabilities {
                if !valid_text(capability) || !capabilities.insert(capability) {
                    return Err(ProjectPackageLockError::InvalidText);
                }
            }
            if !plugins.insert(&entry.plugin_id) {
                return Err(ProjectPackageLockError::DuplicatePlugin);
            }
        }
        if !valid_authority(&self.authority) {
            return Err(ProjectPackageLockError::InvalidAuthority);
        }
        Ok(())
    }

    /// 要求当前对象的条目与能力顺序已经规范化。
    /// 比较的是重新编码后的对象；输入 JSON 的空白、键顺序和转义拼写不参与比较。
    ///
    /// # 返回与错误
    ///
    /// 成功表示当前对象的条目和能力顺序与规范编码一致，且通过
    /// [ProjectPackageLock::canonical_bytes] 的形状与字节预算检查。
    /// 顺序不同返回 [ProjectPackageLockError::NonCanonicalWire]；
    /// 规范编码过程的错误继续传播，当前对象重新编码失败返回
    /// [ProjectPackageLockError::Encoding]。
    ///
    /// # 调用约束
    ///
    /// 适用于已经构造或解码的对象，借用期间不修改它。比较两个重新编码结果，
    /// 因此无法要求原输入 JSON 使用某种空白、字段顺序或转义拼写。
    /// 需要保留这些原始字节特征的消费入口应在解码前另行检查。
    ///
    /// # 参考
    ///
    /// [ProjectPackageLock::decode_canonical]、[ProjectPackageLockState::validate]。
    pub fn require_canonical_wire(&self) -> Result<(), ProjectPackageLockError> {
        let canonical = self.canonical_bytes()?;
        let actual = serde_json::to_vec(self).map_err(|_| ProjectPackageLockError::Encoding)?;
        (actual == canonical)
            .then_some(())
            .ok_or(ProjectPackageLockError::NonCanonicalWire)
    }

    /// 从有界 JSON 输入解码包锁，并要求解码后的对象已经按规范排序。
    ///
    /// # 参数
    ///
    /// `bytes` 是借用的完整锁主体 JSON 输入，长度最多
    /// [MAX_PROJECT_PACKAGE_LOCK_BYTES]。此方法不保留对输入缓冲区的借用。
    ///
    /// # 返回与错误
    ///
    /// 成功返回拥有字段数据的 [ProjectPackageLock]，通过形状、规范编码容量和
    /// 对象顺序检查。原始输入超长返回 [ProjectPackageLockError::Capacity]；
    /// JSON 语法、未知字段或字段类型解码失败返回 [ProjectPackageLockError::Encoding]；
    /// 其他错误来自 [ProjectPackageLock::require_canonical_wire]。
    ///
    /// # 调用约束
    ///
    /// 此方法不比较原 JSON 字节与规范输出；可接受不影响解码对象的空白、
    /// 键顺序和等价转义。消费入口仍须核对项目身份、清单摘要、目标及来源。
    ///
    /// # 参考
    ///
    /// [ProjectPackageLock::canonical_bytes]、[ProjectGuid]、[ProjectManifestDigest]。
    pub fn decode_canonical(bytes: &[u8]) -> Result<Self, ProjectPackageLockError> {
        if bytes.len() > MAX_PROJECT_PACKAGE_LOCK_BYTES {
            return Err(ProjectPackageLockError::Capacity);
        }
        let lock: Self =
            serde_json::from_slice(bytes).map_err(|_| ProjectPackageLockError::Encoding)?;
        lock.require_canonical_wire()?;
        Ok(lock)
    }

    /// 查询是否没有包选择条目；空选择仍可形成有效 `Present` 和真实共享摘要。
    ///
    /// # 返回与约束
    /// 仅在 `entries` 为空时返回 `true`；消费入口仍须执行
    /// [ProjectPackageLock::validate] 及其身份、目标和来源核对。
    pub fn is_empty_selection(&self) -> bool {
        self.entries.is_empty()
    }
}

impl ProjectPackageLockState {
    /// 保留传入锁并派生共享摘要；供要求规范顺序的消费者使用前，应先排序条目与能力。
    /// 摘要在副本上归一化，不会改写状态内的原锁。
    ///
    /// # 参数
    ///
    /// `lock` 是按值传入的包锁；成功时原对象移入 `Present`。
    /// 此方法不会将原对象中的条目或能力排序。
    ///
    /// # 返回与错误
    ///
    /// 成功返回 V1 状态及 [ProjectPackageLock::digest] 派生的共享摘要。
    /// 摘要计算失败时返回其错误；输入锁已经被消费，不随错误返回。
    /// 合法但无序的锁可以构造成功，因为摘要只规范化副本；
    /// 随后 [ProjectPackageLockState::validate] 仍会以 `NonCanonicalWire` 拒绝该状态。
    ///
    /// # 调用约束
    ///
    /// 要求立即消费或通过状态校验时，调用者应先准备规范排序的锁对象。
    /// 构造成功不证明项目身份或目标与请求匹配，也不授予权限。
    ///
    /// # 参考
    ///
    /// [ProjectPackageLock::canonical_bytes]、[ProjectPackageLock::decode_canonical]、
    /// [ProjectPackageLockState::is_present]。
    pub fn present(lock: ProjectPackageLock) -> Result<Self, ProjectPackageLockError> {
        let digest = lock.digest()?;
        Ok(Self::Present {
            schema_version: PROJECT_PACKAGE_LOCK_STATE_SCHEMA_VERSION_V1,
            lock: Box::new(lock),
            digest,
        })
    }

    /// 校验状态协议，并在有锁时核对对象顺序与共享摘要关联。
    ///
    /// # 返回与错误
    ///
    /// `Present` 先检查外层协议版本，再要求锁通过
    /// [ProjectPackageLock::require_canonical_wire]，最后重算共享摘要并与 `digest`
    /// 逐字比较。版本不支持返回 [ProjectPackageLockError::UnsupportedSchema]；
    /// 摘要不匹配返回 [ProjectPackageLockError::NonCanonicalWire]；内部锁错误继续传播。
    /// `Unavailable` 只要求状态版本正确，因此合法不可用状态也返回成功。
    ///
    /// # 调用约束
    ///
    /// 此方法同步借用状态，不排序、不改写摘要，也不检查预期项目身份或目标。
    /// 要求可用锁的入口在本方法成功后仍须选择 `Present` 分支。
    /// 状态内容发生修改后应重算摘要并重新校验。
    ///
    /// # 参考
    ///
    /// [ProjectPackageLockState::present]、[ProjectPackageLockState::is_present]、
    /// [ProjectPackageLock::digest]。
    pub fn validate(&self) -> Result<(), ProjectPackageLockError> {
        match self {
            Self::Present {
                schema_version,
                lock,
                digest,
            } => {
                if *schema_version != PROJECT_PACKAGE_LOCK_STATE_SCHEMA_VERSION_V1 {
                    return Err(ProjectPackageLockError::UnsupportedSchema);
                }
                lock.require_canonical_wire()?;
                if lock.digest()? != *digest {
                    return Err(ProjectPackageLockError::NonCanonicalWire);
                }
                Ok(())
            }
            Self::Unavailable { schema_version, .. } => (*schema_version
                == PROJECT_PACKAGE_LOCK_STATE_SCHEMA_VERSION_V1)
                .then_some(())
                .ok_or(ProjectPackageLockError::UnsupportedSchema),
        }
    }

    /// 查询是否携带 `Present`，供已校验状态的消费入口选择分支。
    ///
    /// # 返回与约束
    /// 空选择的 `Present` 也返回 `true`；该查询不校验版本、顺序或摘要，
    /// 消费前仍须使用 [ProjectPackageLockState::validate]。
    pub fn is_present(&self) -> bool {
        matches!(self, Self::Present { .. })
    }
}

/// 检查单项能力文本是否可进入包锁；集合唯一性由锁校验另行处理。
///
/// # 参数与返回
///
/// `value` 是借用的能力文本。仅当它非空、不超过
/// [MAX_PROJECT_PACKAGE_LOCK_TEXT] 个 UTF-8 字节、没有首尾空白和控制字符时
/// 返回 `true`；不裁剪或改写输入，也不查询能力注册或授权。
///
/// # 参考
///
/// [ProjectPackageLock::validate]、[ProjectPackageLockEntry]。
fn valid_text(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_PROJECT_PACKAGE_LOCK_TEXT
        && value.trim() == value
        && !value.chars().any(char::is_control)
}

/// Match the existing package validator: three ASCII decimal segments, no
/// leading zeros (except zero itself), each representable by u32. This is
/// intentionally narrower than full SemVer because the package service
/// contract has no prerelease/build grammar.
///
/// 检查包锁使用的受限三段版本文本。
///
/// # 参数与返回
///
/// `value` 是借用的版本文本。只有恰好三个点分段、每段均为 ASCII 十进制数字、
/// 可表示为 `u32`，且除单独的 `0` 外没有前导零时返回 `true`。
/// 可接受的单段最大值为 `4294967295`；空段、符号、空白、预发布或构建后缀
/// 均不满足此契约。失败返回 `false`，不提供分段错误或修改输入。
///
/// # 使用边界
///
/// 此谓词规定包锁的文本域，不证明相应版本已发布、已安装或适合当前目标。
/// [ProjectPackageLock::validate] 将失败映射为
/// [ProjectPackageLockError::InvalidVersion]。
pub fn valid_package_version(value: &str) -> bool {
    let mut segments = value.split('.');
    let valid = segments.by_ref().all(|segment| {
        !segment.is_empty()
            && (segment == "0" || !segment.starts_with('0'))
            && segment.bytes().all(|byte| byte.is_ascii_digit())
            && segment.parse::<u32>().is_ok()
    });
    valid && value.split('.').count() == 3
}

/// 检查来源见证的五个字段均具备锁协议要求的摘要文本形状。
///
/// # 参数与返回
///
/// `value` 是借用的 [ProjectPackageLockAuthority]；只有所有字段均通过
/// [is_digest] 时返回 `true`。`build_set_id` 在这里同样按摘要文本检查。
/// 此方法不重新计算文件或库存摘要，也不验证来源、签名或服务端授权。
///
/// # 参考
///
/// [ProjectPackageLock::validate]。
fn valid_authority(value: &ProjectPackageLockAuthority) -> bool {
    is_digest(&value.index_sha256)
        && is_digest(&value.policy_sha256)
        && is_digest(&value.build_set_id)
        && is_digest(&value.capability_digest)
        && is_digest(&value.provider_revision_digest)
}

/// 检查插件标识是否满足包锁的点分名称契约。
///
/// # 参数与返回
///
/// `value` 是借用的插件标识，非空且至多
/// [MAX_PROJECT_PACKAGE_LOCK_TEXT] 个字节。各点分段不能为空，须以小写
/// ASCII 字母开始，其余字节只能为小写字母、数字或下划线；
/// 段不能以下划线结尾或含连续下划线。满足全部条件时返回 `true`。
///
/// 此谓词不改变输入，不要求名称含点，也不确认插件存在。
/// 整个锁的插件唯一性由 [ProjectPackageLock::validate] 检查。
fn valid_plugin_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_PROJECT_PACKAGE_LOCK_TEXT
        && value.split('.').all(|segment| {
            !segment.is_empty()
                && segment
                    .as_bytes()
                    .first()
                    .is_some_and(u8::is_ascii_lowercase)
                && !segment.ends_with('_')
                && !segment.contains("__")
                && segment
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
        })
}

/// 要求包 UUID 文本能解析，且与解析后 UUID 的规范显示结果一致。
///
/// # 参数与返回
///
/// `value` 是借用的包标识。只有 UUID 解析成功且规范小写连字符表示
/// 逐字等于输入时返回 `true`；不改写非规范拼写。这里没有额外限制 UUID
/// 版本或拒绝 nil，因此不能套用 [ProjectGuid] 的非 nil 约束。
///
/// # 参考
///
/// [ProjectPackageLockEntry]、[ProjectPackageLock::validate]。
fn canonical_uuid(value: &str) -> bool {
    uuid::Uuid::parse_str(value).is_ok_and(|id| id.to_string() == value)
}

/// 检查发布修订是否使用正 `u64` 的规范十进制文本。
///
/// # 参数与返回
///
/// `value` 是借用的发布修订；须能解析为大于零的 `u64`，且转回十进制
/// 字符串后逐字等于输入才返回 `true`。因此零、前导零、正号和溢出值无效。
/// 此谓词不证明修订存在，也不提供修订间的新旧比较。
///
/// # 参考
///
/// [ProjectPackageLockEntry]、[ProjectPackageLock::validate]。
fn canonical_revision(value: &str) -> bool {
    value
        .parse::<u64>()
        .ok()
        .filter(|revision| *revision > 0 && revision.to_string() == value)
        .is_some()
}

/// 检查锁字段是否为固定长度的小写十六进制摘要文本。
///
/// # 参数与返回
///
/// `value` 是借用文本；恰有 64 字节，且每个字节都是 `0` 至 `9` 或
/// `a` 至 `f` 时返回 `true`。不接受大写或前缀，也不重算摘要、确认算法或
/// 证明字段关联到可信内容。
///
/// # 参考
///
/// [valid_authority]、[ProjectPackageLock::validate]。
fn is_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[cfg(test)]
#[path = "tests/package_lock.rs"]
mod tests;
