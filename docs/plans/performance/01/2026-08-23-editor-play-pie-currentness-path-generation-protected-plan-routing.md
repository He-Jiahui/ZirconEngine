---
source_report:
  - docs/plans/performance/01/2026-08-23-editor-play-pie-currentness-path-generation-revalidation.md
doc_type: protected-plan-routing
status: routing_pending
---

# Editor Play/PIE currentness与路径generation受保护计划路由（2026-08-23）

- `docs/plans/performance/pending.md`：`core/play/**` current从37更新为**39/39、5,261行、56 tests**；current Cargo、F4
  start/stop/crash/cancel、WPR/xperf/allocator/power/RenderDoc仍open。本Session不直接编辑受保护ledger。
- Editor04 + Plan02 M4：保留Building失败回Edit、typed error、RelPath与NativePluginHostHandle；继续以单一
  `PlaySessionAuthority`硬切同步controller/backend/plugin traits和partial terminal owner。
- Editor04 + Runtime03/10/11：project open generation拥有`ResolvedProjectPath + ProjectPaths`；Play request只传immutable
  identity/handle。当前filesystem project-root resolution/Play request=2，终态per request=0、per project open generation<=1。
- Editor04 + Runtime11：World projection/JSON、snapshot write/fsync、plugin load/enter/exit、spawn/pipe/reap/cleanup全部成为锁外、
  generation-checked ticket；UI/shell/controller/backend锁内只发布/提交O(1)状态。
- Editor14 test infrastructure：Play/project path fixtures不得使用隐式`std::env::temp_dir()`落C盘；受管测试注入D/E/F fixture root，
  记录创建与清理receipt。当前这些Rust tests未运行，不产生C盘artifact。
- `docs/plans/performance/review.md`：只有39/39 current Cargo、fault matrix、path alias parity、F4 WPR/xperf/RSS/power及首帧
  RenderDoc通过后迁入。本轮不迁移、不提交milestone、不发送完成企微。
