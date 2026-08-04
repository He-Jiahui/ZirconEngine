---
handoff_kind: failure
status: open
created_at: 2026-07-30
updated_at: 2026-08-04
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
- 当前状态：`open / implementation_static_green / independent_second_review_green / managed_validation_pending`。本记录不声明 Cargo 通过或产品验收；动态 keymap service、V2 payload identity 与 User locale hot-apply 的审查问题均已完成前向修复并通过二次审查。

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

Open state：`implementation_static_green / independent_second_review_green / managed_validation_pending`。Editor17 已实现唯一 `SettingsAuthority`、有界 cursor/delta change log、Runtime11 shared bounded persistence lane 与 immutable typed settings snapshot；持久化不在 UI/frame caller 执行。2026-08-04 又把六个内建 key 固化为注册期 slot，snapshot 发布不再解析静态 key，未命中变更复用既有 immutable payload；MRU 已新增 authority 内的 typed mutation，字节预算淘汰会将落后 cursor 明确降级为 snapshot。

2026-08-04 independent second review 的三个 Important 已前向修复并完成再审查，`Critical/Important/Minor = 0/0/0`：`EDITOR_KEYMAP_NAME` 现在解析为动态 authority-backed `EditorKeymapService`，已解析的 manager 与 retained-host keyboard dispatch 会观察后续 override；V2、appearance 与 shell extent 仅以 `Arc<EditorDesignTokens>` payload identity 更新，不再由 authority-local generation 驱动无关 MRU/snap 重投影，也不会跨 authority 错误复用。静态格式和契约检查已通过；尚未进入受管 Cargo、F0 启动或 F4 viewport trace。

2026-08-04 locale hot-apply forward repair 已完成并二次复审 `Critical/Important/Minor = 0/0/0`：`editor.language.locale` 是唯一的 User enum authority（`en`/`zh-CN`）；`EditorContextBuilder` 先以启动快照同步 i18n，再安装锁外 settings subscriber，后续 set/clear 与持久层替换都只传递 immutable snapshot。i18n 以 settings generation 拒绝迟到快照，内部 direct locale setter 已收窄为 crate-local，不能再形成公开的第二 preference 写入口。回归覆盖 builder 的 set/clear 热应用、旧 generation 并发倒灌拒绝，以及 subscriber 在 1 秒内可重入读取 bounded delta。限定 `rustfmt --check`、`git diff --check` 和上述静态契约均通过；Cargo/F0/F4 仍必须由 coordinator 创建 current-source 受管验证，故 handoff 保持 `open`。

未完成的验收边界保持不变：必须完成 current-source 受管 Cargo、F0 启动和 F4 viewport 产品 trace，并恢复完整 M1 manifest 的 attribution/lease 后才能受管提交和写 `fixed-*` return。当前 Session 的 write_scope 不可变，而若干已归属 M1 UI/viewport source 尚未进入该 immutable manifest；这只延后 accepted closeout，不得回滚已集成源码或宣称 Cargo/产品验收通过。

## 产出记录与时间

| 日期 | 切片 | 状态 | 完成项目与验证证据 |
| --- | --- | --- | --- |
| 2026-08-04 | Settings failure forward repair | `implementation_static_green / independent_second_review_green / managed_validation_pending` | 注册期 `BuiltInSettingsSlots` 消除 snapshot 发布期的 static key parse；MRU typed mutation 收回 authority，viewport mutation保持 design token/keymap/MRU Arc payload 复用；条目/字节/年龄预算都使落后 cursor 要求 snapshot。三个二审 Important 均已前向修复：模块服务改为动态 `EditorKeymapService`，V2/appearance/shell extent 改以 token `Arc` identity 投影。User locale 现经 context startup snapshot + lock-external subscriber 热应用，generation 拒绝迟到快照，direct setter 为 crate-local；回归覆盖 set/clear、倒灌和有界 delta 的可重入读取。两轮独立审查最终 `Critical/Important/Minor = 0/0/0`，局部 rustfmt 与静态契约检查通过。未运行 Cargo，受管 F0/F4 和完整 immutable manifest attribution 仍待协调器安排。 |
