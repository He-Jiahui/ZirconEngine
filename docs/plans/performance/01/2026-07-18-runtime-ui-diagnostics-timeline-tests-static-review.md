---
related_code:
  - zircon_runtime/src/ui/tests/pipeline_report.rs
  - zircon_runtime/src/ui/tests/diagnostics.rs
  - zircon_runtime/src/ui/tests/timeline.rs
  - zircon_runtime/src/ui/surface/diagnostics.rs
  - zircon_runtime/src/ui/surface/timeline.rs
  - zircon_runtime/src/ui/surface/surface/rebuild.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - docs/plans/zircon_editor/editor_ui/09-editor-modules-and-design-parity.md
tests:
  - 15 pipeline/debug/timeline tests reviewed
  - render-only pipeline stage skipping present
  - default full command/hit/overdraw/overlay snapshot contract present
  - timeline frame-count retention present but byte budget and zero-clone refresh pending
  - current-source Cargo and 1/1k/10k diagnostic scale counters pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime UI diagnostics/timeline测试逐文件性能静态审查（2026-07-18）

## 范围与覆盖

已完整阅读`zircon_runtime/src/ui/tests/{pipeline_report,diagnostics,timeline}.rs`，共3/3个tracked Rust文件、913行、15个测试。范围覆盖pipeline stage顺序/skip、reflector/render/hit/overdraw/debug JSON、layout route report、canvas provenance及timeline retention/selection。

## 正向pipeline门禁

render-only mutation测试要求Layout/PostLayout/Picking skipped、只执行RenderExtract/BatchPrepare；该语义应保留，并扩展为真实stage visited/reused计数。当前测试先构建完整ECS projection再推schedule mask，未证明产品delta不物化full previous/current snapshot，故仍回链PERF-MVP-278/281。

## PERF-MVP-280：默认完整debug成本被测试固化

测试明确要求`UiSurfaceDebugOptions::default()`生成每command record、每occupied hit cell record、overdraw cells及overlay primitives，并在JSON roundtrip中全部非空。默认snapshot还构建material batch String key/reason、node ids、reject messages与完整layout selections。对于1k/10k nodes或4K窗口，这会产生全树/full-grid CPU和大owned payload；应改为默认summary、显式section request与frame/byte budget，测试需同步区分compact default和opt-in full capture。

## Timeline只有frame cap没有byte cap

timeline store测试证明容量2/3按帧逐出，但每frame持完整`UiSurfaceDebugSnapshot`，`snapshot()`返回summaries加全部retained frames；没有retained/returned bytes、Arc handle或刷新零复制门禁。少量超大overdraw/grid/command snapshot仍可占用巨量RSS，UI每次刷新又深复制历史。EditorUI08/09必须采用Arc artifact、frame+byte双预算与selected-frame按需加载。

## 验收要求

1/1k/10k nodes/commands、1080p/4K、timeline 1/60/600记录enabled sections、cell visits/temp bytes、retained/returned bytes、serialization和CPU p95。默认full sections=0；compact stable snapshot rebuild=0；timeline refresh full-frame clone=0且bytes不越预算。显式完整capture/JSON/overlay内容保持一致。current-source Cargo与产品diagnostic viewer trace完成前，3/3留在`pending.md`。
