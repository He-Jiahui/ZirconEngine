---
handoff_kind: failure
status: open
created_at: 2026-07-23
summary_slug: settings-registry-job-category-quota-migration
origin_plan: docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md
origin_workflow_node: M1.1
fixing_plan: docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
origin_child_dir: docs/plans/zircon_editor/editor/17
fixing_child_dir: docs/plans/zircon_editor/editor/14
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/core/jobs/limits.rs
  - zircon_editor/src/core/jobs/system/mod.rs
  - zircon_editor/src/core/settings/
tests:
  - User quota setting range validation and current-shell persistence
  - EditorJobSystem construction consumes resolved limits
  - invalid quota falls back without admitting zero-capacity jobs
---

# Editor14: JobCategory 配额尚未迁入 SettingsRegistry

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md`
- 来源执行切片：Editor17 M1.1 JobCategory quota User-setting migration
- 修复责任计划：`docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md`
- 交接原因：Editor14 是 jobs admission 与 resolved limits 的唯一权威，Editor17 只提供 Settings schema、作用域与持久化。

## 失败现象与复现证据

`EditorJobLimits` 在 `core/jobs/limits.rs:5-39` 将 Thumbnail=2 与 Export=1 固化为私有常量，`with_runtime_defaults` 再把 Import 绑定 runtime parallelism；`EditorJobSystem::with_scheduler_and_bus` 在 `system/mod.rs:39-59` 直接消费这一对象。Editor17 计划要求类别配额成为 User 设置项，但尚无 SettingsRegistry 定义或热应用路径。

这不是可由 Editor17 单独替换的常量：jobs admission 是 Editor14 的权威，Settings 只能提供 schema、作用域和持久化，不能另建第二个 scheduler limits 真相。

## 最低共享层根因

用户可配置的 category quota 仍固化在 Editor14 私有常量中，SettingsRegistry 没有 typed schema，也没有把 resolved limits 接入唯一 admission owner。

## 架构修复验收

- Editor14 为允许配置的类别定义 User scoped、最小值为 1 的整数 Settings 项；保留 runtime-derived Import 默认逻辑的明确优先级。
- 系统构造与设置变更消费同一已解析 limits；`requires_restart` 语义必须明示，不能在运行队列中无序改写配额。
- 删除硬编码用户默认的平行配置入口，不保留私有文件或环境变量覆盖。
- 覆盖非法 0/负数拒绝、User current-shell round-trip、默认回退、启动/热应用时的 admission 限制。

## 禁止临时方案

- 不得在 `EditorJobSystem` 外新增第二个配额 map。
- 不得把 Settings 值仅用于 UI 显示而继续以常量控制 admission。

## 修复结果与回传

Open state: `Editor14 已完成 typed User quota 的单 registry 启动编排、三态加载诊断、完整 limits 一次解析与 restart-only admission 接线；23 路径精确闭包静态门与最终独立复审 0/0/0 已通过，等待 source-bound 受管 Cargo acceptance 后回传 fixed`。

## 产出记录与时间

| 日期 | 切片 | 状态 | 完成项目与验证证据 |
| --- | --- | --- | --- |
| 2026-07-23 | Editor17 M1.1 -> Editor14 quota migration handoff | open | `DEFAULT_THUMBNAIL_LIMIT`、`DEFAULT_EXPORT_LIMIT` 与 runtime Import 默认仍是唯一 admission 输入；User SettingsRegistry 已可承载值，等待 Editor14 接线与旧入口删除。 |
| 2026-08-10 | Editor14 quota startup forward repair | implementation_complete_second_review_repair_static_green | 新增通用 `SettingsStartup`，由 context composition root 在一个产品 registry 注册四类 User/restart-only quota，再原子加载 User 层并保留 `Loaded/Missing/Invalid`；Builder 使用 `scheduler.parallelism()` 一次解析完整 `EditorJobLimits` 后才构造唯一 JobSystem，删除 `SettingsAuthority::at_startup` 隐式失败吞并、JobSystem 二次 runtime default 与 Play/Export fallback 别名。首轮独立审查 `Critical/Important/Minor = 0/2/0` 指出 settings/jobs 反向所有权环与 untracked manifest 风险；前向修复已把 quota 注册/解析上提到 composition root，并将所有新 owner 文件列入精确候选。Rust 行为测试覆盖生产注册、非法 0/负数/>64/错误类型、Missing/Loaded、runtime-derived 类别和 Context A 不热改/Context B 重启生效；settings owner Python 合同 5/5 及 scoped 静态检查通过，等待修复后二次复审与受管 current-source Cargo gate。 |
| 2026-08-10 | Editor14 quota exact-closure repair | implementation_complete_second_review_closure_repair_static_green | 首次复审后的再审 `Critical/Important/Minor = 0/1/2` 确认生产 ownership 环已关闭，同时指出 split settings owner、旧 `tests.rs` 删除、本地化/definition/registry 输入未完整进入精确候选，以及 settings 测试重复引用 jobs quota。已前向扩展为显式 23 路径闭包，删除 settings-to-jobs 测试依赖；完整闭包的 settings owner Python 合同 5/5、16 个 live Rust 源 rustfmt、23 路径 diff-check 均通过，等待最终独立复审；不吸收外部 owner 的 `settings/io.rs` 物理路径改造。 |
| 2026-08-10 | Editor14 quota final re-review | implementation_complete_second_review_clean_managed_validation_pending | 最终独立复审 `Critical/Important/Minor = 0/0/0`：23 路径候选自包含，settings owner 对 jobs 引用为零，Invalid persisted layer 不会部分污染 registry，Builder 是 quota 注册/解析的唯一 composition root。当前仅缺 source-bound 受管 Cargo acceptance，failure 继续保持 open，不提前生成 fixed return。 |
