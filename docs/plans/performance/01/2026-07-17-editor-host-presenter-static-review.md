---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/presenter/**/*.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/**/*.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/08
reference_sources:
  - dev/slint/internal/renderers/software/lib.rs
  - dev/bevy/crates/bevy_ui_render/src
tests:
  - existing GPU presenter damage/cache/stats tests
  - existing softbuffer region/damage/overlay/pixel tests
  - current-source Windows Cargo and fallback present allocation trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor host presenter逐文件性能静态审查（2026-07-17）

## 范围与覆盖

`zircon_editor/src/ui/retained_host/host_contract/presenter`当前共 **31** 个Rust文件、**1,511** 行，已逐文件阅读 **31/31**：root/GPU/trait 13/13、Softbuffer core/surface I/O 10/10、diagnostics 7/7、Softbuffer tests 1/1。动态GPU/Softbuffer产品trace未完成，目录留在`pending.md`。

## 已有正确边界

GPU presenter按surface cache状态区分bootstrap full stream与damage patch，记录upload/draw/visible/batch计数；resize明确失效cache。Softbuffer保留同尺寸backbuffer，region command stream可只重画并只复制damage rows到surface；现有测试覆盖damage clamp、pixel isolation和overlay扩张。Slint software renderer同样以`DirtyRegion`/previous-frame dirty驱动partial rendering，证明dirty authority应驻留renderer generation而不是presentation副本。

## 热点与计划

- PERF-MVP-149：`planned_present`为设置`host_shell.debug_refresh_rate`每帧clone完整`HostWindowPresentationData`。这会叠加PERF-MVP-147中的全部pane/template/RGBA snapshot成本，即使最终只改变一个overlay string。
- 同一路径为使overlay文本中的painted-pixel计数与扩张damage稳定，最多循环8次并在fallback再格式化一次diagnostics text；没有iteration counter或稳定预算。
- verbose logging在检查`last_logged_presentation`前先格式化完整presentation summary，所以稳定帧仍分配summary String。
- GPU和Softbuffer都调用`build_chrome_command_stream`；它是否在damage path仍遍历全部presentation由相邻`chrome_command_stream` 40文件审查验收，本记录不提前宣称增量成立。

EditorUI08/presenter应把diagnostics overlay作为独立typed draw command或small transient输入，并按presentation generation缓存verbose summary；Softbuffer不得克隆结构snapshot。验收记录1k fallback present的clone bytes、overlay iteration、summary build、painted pixels、region/full counts和pixel parity。
