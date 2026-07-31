---
handoff_kind: failure
status: open
created_at: 2026-07-30
updated_at: 2026-07-30
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

## 来源与状态

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 修复责任：Editor17，消费联动Editor05，阻塞持久化执行复用Runtime11。
- current-source静态证据：`docs/plans/performance/01/2026-07-30-editor-core-settings-static-review.md`。
- 当前状态：`open / implementation_and_managed_validation_pending`。本记录不声明源码修复、Cargo通过或产品验收。

## 失败边界

1. viewport项目吸附步进的单key UI命令clone完整registry，随后同步clone完整Project层、序列化整文档并执行write/fsync/rename/parent-sync；慢盘或大设置集直接进入编辑器调用延迟。
2. EditorManager、retained-host design-token启动入口和viewport分别拥有或加载registry，没有唯一settings authority/generation。
3. `SettingsRegistry::changes`无界增长且未找到生产`drain_changes` consumer；稳定workbench snapshot又反复分配三个静态key并查询多层BTreeMap。
4. current settings text先完整解析为`Value`检查magic，再由generic versioned reader解析；该部分必须复用Editor11/PERF-MVP-570，不建立Editor17私有解析器。

## 要求的架构修复

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

## 禁止方案

- 不得为viewport、EditorManager或retained host各建私有缓存、线程池或settings副本。
- 不得把整库clone移到后台后宣称单key更新已优化；不得用无界channel替代无界`changes`。
- 不得删除fsync/atomic replace以换取速度，除非先重定义并验证durability/crash合同。
- 不得在Editor17复制Editor11 versioned reader或Runtime11 persistence scheduler。
