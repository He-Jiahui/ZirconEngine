---
related_code:
  - zircon_hub/src/projects/mod.rs
  - zircon_hub/src/projects/create_project_request.rs
  - zircon_hub/src/projects/recent_project.rs
  - zircon_hub/src/projects/shared_recent_projects.rs
  - zircon_hub/src/projects/validation.rs
implementation_files:
  - zircon_hub/src/projects
plan_sources:
  - docs/plans/zircon_hub/04-settings-draft-and-source-engine.md
tests:
  - zircon_hub/tests/project_management_contract.rs
  - zircon_hub/tests/project_workflow_contract.rs
  - zircon_hub/tests/project_path_scope_contract.rs
doc_type: api-reference
---

# 项目创建、打开与最近项目

本页说明 `zircon_hub::projects` 的项目入口。它负责识别项目根目录、校验 `zircon-project.toml`、构造创建请求、维护最近项目列表，并为 Tauri 层提供可序列化的项目事实。它不负责加载 Runtime 世界，也不替代编辑器的场景打开逻辑。

## 适用边界

| 入口 | 所属层 | 是否建议外部调用 | 责任 |
| --- | --- | --- | --- |
| `CreateProjectRequest` | 项目请求值对象 | 是 | 规范化创建参数并计算目标目录 |
| `project_template_catalog` | Hub 模板目录 | 是 | 返回编译期启用的模板描述 |
| `validate_project_root` | 路径校验 | 是 | 判断目录是否可作为 Zircon 项目 |
| `RecentProject` | 历史条目 | 是 | 保存 manifest 摘要、绝对路径与时间 |
| `load_shared_recent_projects*` | Hub 状态 | 谨慎 | 从共享历史文件读取并过滤条目 |
| `create_project` Tauri command | UI 编排 | 否 | 内部组合模板复制、引擎绑定和启动 |

## 项目根目录契约

项目根目录是包含 `zircon-project.toml` 的目录。路径比较通过 `normalize_project_root` 与 `project_paths_match` 完成；调用者不应自己用字符串比较路径，也不应把 `target`、包输出目录或源码引擎目录当成项目根。

```text
选择目录
   |
   v
normalize_project_root
   |
   +--> validate_project_root --> Valid / MissingManifest / InvalidManifest / NotDirectory
   |
   +--> metadata key + recent history lookup
   |
   `--> editor launch request
```

`validate_project_root(path)` 返回 `ProjectValidation`，调用者应先处理结果，再读取 manifest。不要在验证失败后继续猜测项目名称或默认场景。

## `CreateProjectRequest`

```rust
pub struct CreateProjectRequest {
    pub template_id: ProjectTemplateId,
    pub project_name: String,
    pub parent_dir: PathBuf,
    pub engine_id: Option<String>,
}

impl CreateProjectRequest {
    pub fn new(
        template_id: ProjectTemplateId,
        project_name: impl Into<String>,
        parent_dir: impl Into<PathBuf>,
        engine_id: Option<String>,
    ) -> Self;

    pub fn validate_launch_fields(&self)
        -> Result<(), CreateProjectRequestError>;

    pub fn target_root(&self) -> PathBuf;
}
```

`new` 只构造值对象，不创建目录。`target_root` 根据父目录与规范化项目名计算目标路径；在真正执行创建前必须调用 `validate_launch_fields`。错误通常包括空名称、非法文件名字符、缺少父目录或无效模板。错误类型是 `CreateProjectRequestError`，UI 应把它转换为本地化 `HubMessage`，而不是直接把 Debug 字符串展示给用户。

## 模板目录

```rust
pub struct ProjectTemplateInfo {
    pub id: ProjectTemplateId,
    pub display_name: &'static str,
    pub description: &'static str,
}

pub fn project_template_catalog() -> &'static [ProjectTemplateInfo];
pub fn enabled_project_template_id(id: &str) -> Option<ProjectTemplateId>;
```

模板目录是静态切片，生命周期为 `'static`。UI 应以 `id` 作为提交值，以 `display_name` 作为展示文本。不要把展示文本反向解析为模板 ID，也不要缓存一个可能来自旧版本的枚举字符串。

## 打开既有项目

`RecentProject::from_project_path(path, last_opened_unix_ms)` 会读取 `path/zircon-project.toml`，通过 `ProjectManifestSummary::parse_toml_bytes` 解析摘要。`with_now(path)` 使用 `now_unix_ms()` 写入当前时间；`refresh_summary()` 用磁盘 manifest 更新摘要，但保留原有打开时间。

```rust
use zircon_hub::projects::{validate_project_root, RecentProject};

let root = std::path::PathBuf::from("E:/Projects/Weather");
match validate_project_root(&root) {
    zircon_hub::projects::ProjectValidation::Valid => {
        let item = RecentProject::with_now(&root)?;
        println!("{} -> {}", item.display_name(), item.path.display());
    }
    invalid => eprintln!("project cannot open: {invalid:?}"),
}
```

读取 manifest 失败会返回 `HubError`，包括文件不存在、常规文件限制、UTF-8/TOML 解析失败和格式版本不兼容。打开流程不应自动修复用户文件；应提供“重新选择目录”或“查看 manifest”恢复动作。

## 最近项目合并

`RECENT_PROJECT_LIMIT` 来自共享协议常量，不能在 UI 中硬编码另一个上限。`merge_recent_project_entries` 按规范化路径去重并保留最新打开时间；`reconcile_shared_recent_projects` 负责将磁盘条目与当前有效项目集合合并；snapshot 变体同时返回 generation/快照信息。

```rust
let snapshot = load_shared_recent_projects_snapshot(&history_path)?;
let visible = snapshot.projects()
    .iter()
    .filter(|entry| validate_project_root(&entry.path).is_valid())
    .collect::<Vec<_>>();
```

上例中的 `is_valid` 代表调用者对枚举的匹配逻辑，实际枚举变体以当前源码为准。共享文件损坏时应保留原文件副本，再写入空历史；不可把损坏数据静默覆盖。

## 路径与安全规则

| 规则 | 原因 | 推荐动作 |
| --- | --- | --- |
| 先规范化再比较 | Windows 大小写、分隔符和 UNC 路径差异 | 调用 `project_paths_match` |
| 创建目标不能是已有项目 | 防止模板覆盖用户内容 | 先验证目标不存在 |
| 输出目录不能嵌入项目 | 避免打包递归复制自身 | 使用 `reject_inside_root` 逻辑 |
| manifest 必须是常规文件 | 避免设备文件/FIFO 阻塞 | 通过 Hub 文件 IO 读取 |
| 历史只保存路径和摘要 | 减少敏感信息暴露 | 不写访问令牌和环境变量 |

## 与 Unreal、Godot 的对应关系

Unreal Project Launcher 通常把“项目入口、引擎版本、最近项目”作为独立记录；Zircon Hub 同样把 `RecentProject.summary` 与 `SourceEngineInstall.id` 分离，因此项目移动后可重新绑定引擎。Godot 的项目管理器偏向扫描 `project.godot` 并提供导入/编辑按钮；Zircon Hub 的校验更严格，manifest 解析失败会阻止启动，并保留结构化恢复消息。

## 失败恢复清单

1. `NotDirectory`：重新选择目录，不创建隐式目录。
2. `MissingManifest`：确认目录层级，不能只凭文件夹名称判断。
3. 解析失败：保留原 manifest，显示诊断位置，修复后重新 `refresh_summary`。
4. 最近项目过期：移除该条目并保留其他条目，不能影响当前已打开项目。
5. 目标已存在：改用新名称或明确的空目录，禁止递归覆盖。

## 测试关注点

项目合同测试覆盖创建字段、项目页面、路径作用域和历史行为。新增调用者至少应测试：路径大小写等价、缺失 manifest、损坏 TOML、重复历史、迁移后刷新摘要和目标目录冲突。

## 相关 API

- [Hub 总览](../zircon-hub.md)
- [导出与项目打包](../export-and-packaging.md)
- [Hub 状态与命令](hub-state-commands.md)
- [项目工作流合同测试](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_hub/tests/project_workflow_contract.rs)
