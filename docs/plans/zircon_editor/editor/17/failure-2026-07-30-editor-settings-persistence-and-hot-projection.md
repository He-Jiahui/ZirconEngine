---
handoff_kind: failure
status: open
created_at: 2026-07-30
updated_at: 2026-08-08
summary_slug: editor-settings-persistence-and-hot-projection
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor/17
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/core/settings
  - zircon_editor/src/ui/host/editor_manager.rs
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_accessors.rs
  - zircon_editor/src/ui/workbench/snapshot/data/editor_state_snapshot_build.rs
tests:
  - cargo test -p zircon_editor --lib core::settings --locked --jobs 1 -- --test-threads=1
---

# Editor17：设置持久化与高频投影性能交接

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：2026-07-30 editor core settings current-source 静态审阅。
- 修复责任计划：`docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md`
- 交接原因：设置 authority、无界 change retention、同步持久化与高频 snapshot 投影跨越 Editor17/Editor05/Runtime11 边界，不能由 Performance01 在局部性能切片中建立第二套缓存或调度器。
- current-source 静态证据：`docs/plans/performance/01/2026-07-30-editor-core-settings-static-review.md`。
- 当前状态：`open / implementation_static_green / independent_second_review_green / external_handoff_open / managed_validation_rebind_required`。本记录不声明 Cargo 通过或产品验收；动态 keymap service、V2 payload identity 与 User locale hot-apply 的审查问题均已完成前向修复并通过二次审查，本轮 settings presentation 硬切的最终独立复审也为 `Critical/Important/Minor = 0/0/0`。2026-08-05 的 registry/snapshot/authority 所有权硬切及其 project-layer lifecycle 前向修复均已二审，必须由新的受管 snapshot 验证；Editor10 的第二 authority 入口 hard cut 仍是外部回传依赖。

## 失败现象与复现证据

1. viewport项目吸附步进的单key UI命令clone完整registry，随后同步clone完整Project层、序列化整文档并执行write/fsync/rename/parent-sync；慢盘或大设置集直接进入编辑器调用延迟。
2. EditorManager、retained-host design-token启动入口和viewport分别拥有或加载registry，没有唯一settings authority/generation。
3. `SettingsRegistry::changes`无界增长且未找到生产`drain_changes` consumer；稳定workbench snapshot又反复分配三个静态key并查询多层BTreeMap。
4. current settings text先完整解析为`Value`检查magic，再由generic versioned reader解析；该部分必须复用Editor11/PERF-MVP-570，不建立Editor17私有解析器。

## 最低共享层根因

设置定义、运行时值、change history、持久化调度与 UI snapshot 没有围绕单一 generation authority 收敛。各消费端因此复制 registry 或重新解析/投影，单 key 更新又把整库 clone、完整序列化与同步文件系统耐久化串进 UI 调用路径；无界 `changes` 同时把短期事件日志错误地变成长期存储。

## 架构修复验收

- Editor17拥有唯一settings authority：注册定义后发布typed key slot与immutable generation snapshot；EditorManager、retained host、viewport和设置页只消费同一authority或delta，不复制完整registry作为第二真相。
- no-op set不得增长revision/event；change delivery使用有entry+bytes+age预算的cursor/delta，不能以无界内部Vec保留整个编辑器生命周期。
- 持久化只接收typed changed key/scope/generation；Runtime11共享bounded lane按scope/key合并latest generation，在worker上构建最终文档并复用共享atomic writer。UI/frame caller不执行filesystem；flush/shutdown/error/retry/cancel有明确ticket结果。
- Editor05的snapshot投影只读取预解析typed slot或同generation共享值；稳定settings generation不得再分配`SettingsKey(String)`或遍历definition/session/project/user树。
- startup的user/project source每generation至多读取一次；严格envelope和typed payload遍数遵守PERF-MVP-570。

## 验收

- keys/definitions `1/1k/100k`、events `1/1k/1M`、snapshots `60/120 Hz`、values `0/1KiB/1MiB`、filesystem `0/10ms/2s`、writers/consumers `1/16`。
- 记录registry owners、file reads/decode passes、full clone bytes、key/String alloc、BTree probes、journal/queue entries+bytes+age、writes/fsync、caller wall、RSS与p50/p95。
- 必须满足authority=1、UI caller filesystem wall=0、single-key full-registry clone bytes=0、stable snapshot key alloc/probe=0、journal/queue内存硬有界、no-op event=0；precedence、keymap、design tokens、MRU、snap、restart、crash old/new与shutdown语义保持一致。
- 通过frontmatter focused current-source managed gate，并以F0启动和F4 viewport产品trace证明；静态rustfmt或单元测试不能替代规模与调用线程证据。

## 禁止临时方案

- 不得为viewport、EditorManager或retained host各建私有缓存、线程池或settings副本。
- 不得把整库clone移到后台后宣称单key更新已优化；不得用无界channel替代无界`changes`。
- 不得删除fsync/atomic replace以换取速度，除非先重定义并验证durability/crash合同。
- 不得在Editor17复制Editor11 versioned reader或Runtime11 persistence scheduler。

## 修复结果与回传

Open state：`implementation_static_green / independent_second_review_external_handoff_open / managed_validation_rebind_required`。Editor17 已实现 authority-owned cursor/delta change log、Runtime11 shared bounded persistence lane 与 immutable typed settings snapshot；持久化不在 UI/frame caller 执行。2026-08-04 又把六个内建 key 固化为注册期 slot，snapshot 发布不再解析静态 key，未命中变更复用既有 immutable payload；MRU 已新增 authority 内的 typed mutation，字节预算淘汰会将落后 cursor 明确降级为 snapshot。2026-08-08 current-source second review 发现 project document owner 仍对 production surface 暴露会自行构造 `SettingsAuthority` 的 legacy entry，已按最低 owner 路由 Editor10 `failure-2026-08-08-project-document-settings-authority-legacy-entry.md`；该 forward handoff 未回滚 Editor17 实现，但其未 return 前不得声明全图 authority=1。

2026-08-04 independent second review 的三个 Important 已前向修复并完成再审查，`Critical/Important/Minor = 0/0/0`：`EDITOR_KEYMAP_NAME` 现在解析为动态 authority-backed `EditorKeymapService`，已解析的 manager 与 retained-host keyboard dispatch 会观察后续 override；V2、appearance 与 shell extent 仅以 `Arc<EditorDesignTokens>` payload identity 更新，不再由 authority-local generation 驱动无关 MRU/snap 重投影，也不会跨 authority 错误复用。静态格式和契约检查已通过；尚未进入受管 Cargo、F0 启动或 F4 viewport trace。

2026-08-04 locale hot-apply forward repair 已完成并二次复审 `Critical/Important/Minor = 0/0/0`：`editor.language.locale` 是唯一的 User enum authority（`en`/`zh-CN`）；`EditorContextBuilder` 先以启动快照同步 i18n，再安装锁外 settings subscriber，后续 set/clear 与持久层替换都只传递 immutable snapshot。i18n 以 settings generation 拒绝迟到快照，内部 direct locale setter 已收窄为 crate-local，不能再形成公开的第二 preference 写入口。回归覆盖 builder 的 set/clear 热应用、旧 generation 并发倒灌拒绝，以及 subscriber 在 1 秒内可重入读取 bounded delta。限定 `rustfmt --check`、`git diff --check` 和上述静态契约均通过；Cargo/F0/F4 仍必须由 coordinator 创建 current-source 受管验证，故 handoff 保持 `open`。

未完成的验收边界保持不变：必须完成 current-source 受管 Cargo、F0 启动和 F4 viewport 产品 trace，并恢复完整 M1 manifest 的 attribution/lease 后才能受管提交和写 `fixed-*` return。当前 Session 的 write_scope 不可变，而若干已归属 M1 UI/viewport source 尚未进入该 immutable manifest；这只延后 accepted closeout，不得回滚已集成源码或宣称 Cargo/产品验收通过。

2026-08-05 按结构约定完成 `core/settings/tests.rs` 硬切为 folder-backed `tests/{mod,registry,persistence}.rs`：30 个测试入口保持一一对应，公共 fixture 和临时目录助手收束在私有父测试模块，未新增兼容包装或生产 API。第一次独立审查发现子模块经 `super::` 使用的 `SettingsPersistenceSubmitError` 未在父模块重导出；现已前向恢复该共享 import，复审最终 `Critical/Important/Minor = 0/0/0`。随后 current-source 静态审计报告 `editor_manager.rs` 与 retained-host `app.rs` 的格式漂移，已以 `rustfmt` 前向收敛；7 路径 `rustfmt --check`、`git diff --check`、30 测试入口和共享 import 守卫均再次通过。`app.rs` 同时承载未提交的 M3 logging boundary work，故该 7 路径检查仅证明当前源码卫生，不替代完整 M1 immutable manifest attribution。最新独立复审还发现 test-tree manifest 未绑定本 failure record；记录现已加入 manifest，最终 re-review `Critical/Important/Minor = 0/0/0`。本条仅记录实现和静态证据，不改变 failure 的 `open / managed_validation_pending` 状态。

2026-08-05 settings presentation hard cut 已完成实现与静态门：`SettingDefinition` 删除公共 slash-separated `category_path: String`，改为私有 `SettingsPresentation`，其中 label、description 和非空 category path 都只能是验证过的 locale-neutral `settings.*` key；空段和尾点均被拒绝。七个默认设置与四个 Editor job quota 注册点均已迁移，英中 bundle 补齐 31 个 key，定义/配额测试分别验证每个 embedded locale 都能解析 label、description 与 category。限定 `rustfmt --check`、范围内 `git diff --check`、31 key 双语覆盖和 failure graph 检查（564 artifacts / 0 errors）通过；最终独立复审 `Critical/Important/Minor = 0/0/0`，确认已删除旧私有字段构造、11 个 builtin 均逐 bundle 直接验证、15-path M1 manifest 覆盖生产/配额/资产范围，且主计划不再陈述旧 API。尚未运行本地 Cargo，也不改变 failure 的 `open` 状态。两份 bundle 同时属于当前 M3 candidate，故不得伪造独立 M1 immutable manifest；协调器必须为 M1/M3 构造新的联合 current-source snapshot 后再安排受管 gate。

## 产出记录与时间

| 日期 | 切片 | 状态 | 完成项目与验证证据 |
| --- | --- | --- | --- |
| 2026-08-04 | Settings failure forward repair | `implementation_static_green / independent_second_review_green / managed_validation_pending` | 注册期 `BuiltInSettingsSlots` 消除 snapshot 发布期的 static key parse；MRU typed mutation 收回 authority，viewport mutation保持 design token/keymap/MRU Arc payload 复用；条目/字节/年龄预算都使落后 cursor 要求 snapshot。三个二审 Important 均已前向修复：模块服务改为动态 `EditorKeymapService`，V2/appearance/shell extent 改以 token `Arc` identity 投影。User locale 现经 context startup snapshot + lock-external subscriber 热应用，generation 拒绝迟到快照，direct setter 为 crate-local；回归覆盖 set/clear、倒灌和有界 delta 的可重入读取。两轮独立审查最终 `Critical/Important/Minor = 0/0/0`，局部 rustfmt 与静态契约检查通过。未运行 Cargo，受管 F0/F4 和完整 immutable manifest attribution 仍待协调器安排。 |
| 2026-08-05 | Settings test-tree and presentation hard cut | `implementation_static_green / independent_second_review_green / managed_validation_pending` | 1170 行 `core/settings/tests.rs` 已删除，替换为 `tests/mod.rs`（82 行）、`tests/registry.rs`（648 行）和 `tests/persistence.rs`（445 行）；对照 HEAD 验证 30/30 测试、35/35 函数名无新增或遗漏，逐测试体的归一化比对无行为差异，`rustfmt --check`、范围内 `git diff --check` 通过。结构审计仍确认 `oversized_production_file_count=0`；全局剩余 UI v2 owner 与重复 UI 测试树迁移债务不在本 scope。首次独立审查发现 `SettingsPersistenceSubmitError` 在拆分后未被父测试模块导入，已前向修复并复审 `Critical/Important/Minor = 0/0/0`；后续 static audit 收敛 `editor_manager.rs` 与 retained-host `app.rs` 的格式漂移，后者含有未提交 M3 logging boundary work，故检查不替代完整 M1 attribution。最新独立复审发现 manifest 漏绑本 failure record，现已前向补齐并最终复审 `Critical/Important/Minor = 0/0/0`。本轮再删除 `SettingDefinition.category_path: String` 和默认/配额注册的英文 slash path，改以私有 `SettingsPresentation` 存储 label、description 与 category localization key；七个默认设置和四个 quota 在所有 embedded locale 的解析由新回归覆盖，31 个 key 在英中 bundle 均存在。二审新增的私有字段测试、fallback-only locale coverage、M1 manifest 范围和主计划旧 API 四项问题均已前向修复，最终复审 `Critical/Important/Minor = 0/0/0`；限定静态门和 handoff graph（564/0）通过，未运行本地 Cargo。两份本地化资产与 M3 candidate 共享，后续仅可由协调器创建新的联合 immutable snapshot。 |
| 2026-08-05 | Settings registry/snapshot/authority owner hard cut | `implementation_static_green / independent_second_review_pending / managed_validation_rebind_required` | 865 行 `core/settings/registry.rs` 已前向拆为 265 行 registry（definitions/layers/precedence/change log）、275 行 snapshot（built-in typed slots/immutable projection）和 339 行 authority（publication/subscriber/project-layer lifecycle）；`mod.rs` 仅重导出现有 public surface，不保留旧路径 alias 或兼容 wrapper。TDD 结构守卫先暴露 owner 缺失和超 800 行，拆分后新增 owner/entry-point guards 与既有 M3.1/M2 guards 共 17/17 静态契约、`py_compile`、限定 `rustfmt --check` 和范围 `git diff --check` 通过。此变更未被先前 M1 immutable manifest 包含，故 manifest 已前向扩展为包含两个新 source owner；未运行本地 Cargo，独立二审和 current-source managed validation 均需重新绑定，failure 保持 open。 |
| 2026-08-08 | M1 authority uniqueness second-review routing | `external_handoff_open / managed_validation_rebind_required` | 复审确认 active project-open 路径已经注入 `EditorContext` 的共享 authority，但 `EditorProjectDocument::load_from_project` 仍以公开 production API 自行构造 `SettingsAuthority::with_defaults()`；其当前 call-site 均为 crate tests，说明应 hard-cut 为 test-only helper，不能保留可复活的第二 authority。最低 owner 为 Editor10 project/document loading，已创建 [Editor10 failure](../10/failure-2026-08-08-project-document-settings-authority-legacy-entry.md)；Editor17 不越权修改该 source，且不将这一外部待修复误写为验证通过。 |
| 2026-08-08 | M1 project-layer lifecycle second-review forward repair | `implementation_static_green / independent_second_review_green / external_handoff_open / managed_validation_rebind_required` | 首轮独立审查发现 project cache mutex 在 `replace_persistent_layer` 的 subscriber 回调期间仍被持有，回调内请求 Project 持久化准备会自锁。已以独立 operation gate 和显式 `transition_in_progress` 状态前向修复：load/clear 在回调前清空 cache 并标记 transition，回调的 `prepare_persistent_layer_for_write(Project, ...)` 明确返回 `Ok(None)`，完成 source replacement 后才安装新 cache；回归同时覆盖 load 与 clear 的 1 秒有界回调。修复后独立复审 `Critical/Important/Minor = 0/0/0`，5 组/19 条静态契约、`py_compile`、限定 `rustfmt --check` 与范围 `git diff --check` 已通过。未运行本地 Cargo；Editor10 legacy entry 的 authority=1 外部 handoff 与 coordinator current-source 验证仍未完成。 |
