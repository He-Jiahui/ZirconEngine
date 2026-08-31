---
related_code:
  - zircon_editor/src/core/i18n
canonical_review:
  - docs/plans/performance/01/2026-08-23-editor-core-i18n-currentness-and-fallback-m0.md
protected_targets:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/pending.md
  - docs/plans/performance/review.md
owner_plans:
  - docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
doc_type: protected-plan-routing
status: update_requested_not_applied
---

# Editor core i18n当前性保护计划路由（2026-08-23）

## 请求Performance01纠正

将`zircon_editor/src/core/i18n/**`记录为M0后7/7 Rust文件、1,098 physical lines、36,717 bytes、
10 tests，ordered path + NUL + raw bytes + NUL SHA256为
`d2941850bc5cb8ce79b289039b5f39da03b83637377de4a6654e19edc96f5163`，状态为
`simple_m0_landed / dynamic_and_structural_pending`。

不要新增独立P0 catalog rewrite。当前2 locales/54 keys没有证明BTreeMap是瓶颈；本轮已把非英文fallback
每次1个English locale owner allocation静态收为0。缺失raw key分配、map/slot算法和future bundle loading
必须先由scale/allocator数据决定。

## 必需的现有任务修正

### PERF-MVP-596

主计划当前只描述Decision generation，没有接纳i18n currentness责任。要求补入：

- `EditorI18nService`拥有唯一non-zero `TextRevision`和typed invalidation cause；
- Decision/Toast/Progress统一projection消费text revision，stable tick翻译、fallback、format和row build为0；
- accepted visible text change最多build/apply一次，no-op/stale settings不增text revision；
- 空generation仍能清UI；不得在notification内建立第二locale counter。

### PERF-MVP-591

- settings仍是locale preference唯一authority，只在unlock后发布一次affected locale slot；
- accepted settings generation只是trigger，canonical locale实际变化才推进text revision；
- stable retained frame不读取settings snapshot、不执行i18n synchronization。

### Editor17与EditorUI08 owner计划

- Editor17保留English borrowed lookup M0、captured-locale consistency、bounded event resync，并定义
  text revision、missing-key entries/bytes预算和唯一external compatibility边界；
- 若没有production plugin/runtime consumer，hard-cut `EditorTopic::i18n()` JSON/fanout；若存在则仅序列化
  同一typed revision/cause，不能建立第二authority；
- EditorUI08保存last-applied text/notification token，每个accepted generation只materialize visible rows
  一次；future bundle/font commit必须reject stale revision并区分locale与bundle invalidation。

## 受保护索引状态请求

- `pending.md`：替换为上述current冻结、canonical review和
  `simple_m0_landed / dynamic_and_structural_pending`。
- `review.md`：在managed Cargo、allocator/lock/scale counters、至少31次F4 WPR CPU/RSS/power以及locale
  change RenderDoc glyph/draw parity通过前，不得加入。

本会话不修改受保护索引、Performance01主计划或owner计划。当前无动态验收里程碑，因此不提交git
commit，也不发送企微量化通知。
