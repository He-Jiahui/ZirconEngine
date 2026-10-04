---
related_code:
  - zircon_editor/src/core/process.rs
  - zircon_editor/src/ui/host/export_process_support
  - zircon_editor/src/ui/host/export_cargo_process.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/execution.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/execution
canonical_review:
  - docs/plans/performance/01/2026-08-23-editor-process-supervision-output-currentness-revalidation.md
protected_targets:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/pending.md
  - docs/plans/performance/review.md
owner_plans:
  - docs/plans/zircon_editor/editor/04-pie-and-simulation.md
  - docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
  - docs/plans/zircon_editor/editor/15-build-export-and-publishing.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
doc_type: protected-plan-routing
status: update_requested_not_applied
---

# Editor进程监管与输出保护计划路由（2026-08-23）

## 请求Performance01纠正

将current切片记录为10/10 Rust文件、3,033 physical lines、103,251 bytes、19 tests、SHA
`21a707cd7a05a3822aefb31a7247f377ae014e524abc94bfd832162221baefe7`，状态为
`partial_output_m0_present / structural_and_dynamic_pending`。

### PERF-MVP-080

删除“Cargo完整stdout/stderr Vec再完整String、terminal full drain”的现状描述。current已使用每流
256 KiB tail和64 KiB terminal chunks。保留为回归与剩余cutover：一个canonical full artifact、
locator/digest/count/tail result、无temporary relay、1 GiB working set与UI delta有界。

### PERF-MVP-091

删除“wizard无界log/tail”的现状描述。current已有512行tail、16 KiB line decoder、streaming artifact
和digest。剩余P0是relay→memory→artifact双写、每行同步UI callback、25 ms sleep polling以及report仍投影
bounded line arrays。要求一个stream owner、bounded UI delta和虚拟化consumer。

### PERF-MVP-639

该ID仍只存在旧路由、未进入主计划。请求接纳为P0：Play/export process authority分裂；Windows Play用
Toolhelp全系统thread scan，export steady cancellation启动`taskkill`；terminate消费唯一owner，Drop可同步
kill/wait，general worker按25/100 ms sleep polling。

目标是Runtime11唯一`ProcessSessionGeneration`和
`terminate -> reap -> pipe close -> artifact cleanup` receipt chain；Windows platform spawn直接持有
CreateProcess process/primary-thread handles并在Job attach后resume；failure可retry；private process worker/thread、
blocking Drop、Toolhelp discovery和`taskkill` steady path全部hard-cut。

## owner计划责任

| plan | required merge |
|---|---|
| Runtime11 | `ProcessSpec/session/generation/receipt`、platform native handles、shared wait/readiness、bounded output/artifact policy和metrics唯一owner |
| Editor14 | general CPU worker不sleep/wait process；output/completion以generation-checked ticket返回；Drop只nonblocking submit cleanup |
| Editor15 | child直接写唯一approved-root canonical artifacts；streaming digest/tail/typed UI deltas；删除temp relay和完整payload report |
| Editor04 | Play复用同一session；tree/pipes/snapshot保留到terminal receipts；cleanup failure显式pending且可retry |

`pending.md`更新current冻结与canonical review；在approved-root managed tests、1 GiB/100-process矩阵、
至少31次F4 WPR CPU/I/O/RSS/power和terminal failure/retry全部通过前不得进入`review.md`。本会话不修改
受保护文件或owner plans，也不触发commit/企微。
