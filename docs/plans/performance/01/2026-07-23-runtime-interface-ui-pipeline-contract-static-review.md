---
related_code:
  - zircon_runtime_interface/src/ui/pipeline
  - zircon_runtime/src/ui/surface/surface/rebuild.rs
  - zircon_editor/src/ui/workbench/debug_reflector/schedule_sections.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
reference_sources:
  - dev/bevy/crates/bevy_diagnostic/src/diagnostic.rs
  - dev/bevy/crates/bevy_dev_tools/src/diagnostics_overlay.rs
tests:
  - zircon_runtime/src/ui/tests/pipeline_report.rs
  - zircon_editor/src/ui/workbench/debug_reflector/schedule_sections_tests.rs
  - current-source Windows UI pipeline tests pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime interface UI pipeline 性能静态审查（2026-07-23）

## 范围与覆盖

`zircon_runtime_interface/src/ui/pipeline/**` 当前源 **6/6** 个 Rust 文件、**328** 行已逐文件阅读，并反查runtime `UiSurfaceRebuildReport::pipeline_report`生产、`UiSurface::surface_frame`发布与Editor debug reflector消费。runtime 3条focused tests和Editor 1条focused test覆盖stage顺序、render-only skip、surface/debug snapshot一致性与可见诊断文本。目录当前无工作区改动，本轮未修改源码。

## 性能结论

- `UiPipelineStage::ORDER`与archived stage表是静态定长数组，counter为Copy flat bag；单次total、lookup与order检查最多10个stage，固定规模本身不是算法瓶颈。
- live report仍以`Vec<UiPipelineStageReport>`持有固定10行，每行再拥有`Vec<UiPipelineDirtyReason>`和`Vec<String>` notes。runtime每次`surface_frame()`调用都重新构造10行；即使stage skipped，仍为静态说明执行`note.to_string()`，dirty reason helper也分别创建Vec。该证据精确补强已有 **PERF-MVP-278**，不重复编号。
- Editor debug reflector随后为missing stages、dirty reasons和counter摘要建立多层临时Vec/String并join。它只应在诊断面板可见且按受控cadence/changed generation刷新，不能把report格式化成本扩散到普通F4 frame。
- `missing_required_stages()`会分配Vec并以10次线性stage lookup形成固定O(10²)，但它只属于诊断查询；先消除稳定generation下整个report的重建/深clone，再由counter证明是否需要定长bitmask或无分配iterator，避免无收益微优化。

## PERF-MVP-278 补充设计与验收

1. rebuild按layout/render/input/window独立generation发布generation-owned `Arc<UiSurfaceFrameData>`；pipeline rows和静态notes只在对应generation改变时生成。普通consumer借用或clone Arc，读取window/rebuild stats不得触发wide frame snapshot。
2. live stage notes采用静态/borrowed descriptor或只在序列化/诊断边界物化；保持当前serde字段和archived stage可读，不引入第二套兼容report。
3. Editor Runtime Diagnostics仅在面板可见、report generation改变或有界刷新cadence时格式化；记录format calls、String/Vec allocations、bytes与pane p95。稳定generation 1,000次frame access要求pipeline report build=0、note/dirty-reason owned bytes=0。
4. 1/1k/10k nodes与input/layout/render/window单域变化分别记录frame-data owners、Arc clones、stage-row builds和invalidations；现有4条focused合同、surface serde与diagnostic text保持等价。

## 参考引擎对照

Bevy `DiagnosticPath::const_new`用borrowed `Cow<'static, str>`和预计算hash保存静态诊断身份，`DiagnosticsStore`按引用查询；其dev-tools diagnostics overlay明确以1秒timer重建显示内容。Zircon采用相同的“静态metadata借用 + 可见诊断受控刷新”原则，但保留自身10-stage ABI和generation-owned surface frame，不照搬Bevy ECS资源形状。

current-source Cargo、stable-generation allocation counter与F4 Runtime Diagnostics产品trace未完成，因此该目录继续保留在 `pending.md`，不进入 `review.md`。
