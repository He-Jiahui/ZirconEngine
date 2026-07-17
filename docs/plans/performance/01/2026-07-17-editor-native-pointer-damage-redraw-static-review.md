---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/chrome_damage.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/chrome_damage/**/*.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/close_prompt_damage.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/close_prompt_damage/**/*.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/pane_button_damage.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/pane_button_damage/**/*.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/redraw_result.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/redraw_result/**/*.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/resize_damage.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/tab_drag_damage.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/tab_drag_damage/**/*.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/template_hover_damage.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/viewport_toolbar_damage.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/viewport_toolbar_damage/**/*.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
reference_sources:
  - dev/slint/internal/core/item_rendering.rs
tests:
  - retained native pointer damage and redraw tests
  - current-source Windows Cargo pending
  - separated-region and 1000-event damage-area trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor native pointer damage/redraw逐文件性能静态审查（2026-07-17）

## 范围与覆盖

`chrome_damage`、`close_prompt_damage`、`pane_button_damage`、`redraw_result`、`tab_drag_damage`、`viewport_toolbar_damage`及`resize_damage.rs`、`template_hover_damage.rs`共 **57/57** 个Rust文件、**843** 行已逐文件阅读。当前源Cargo、分离damage面积和产品event trace未完成，因此仍留在`pending.md`。

## 已有正确边界

Move在interaction state未变时返回idle；Viewport不因pointer state主动重绘；Hierarchy hover可只覆盖old/new row；template hover合并old/new frame；close prompt、resize、tab drop和toolbar对不可见frame返回None并保留full-frame fallback。各模块没有阻塞、I/O、线程、队列或复杂递归，damage计算主要是常数几何和少量模型查找。

## 热点与既有计划归属

PERF-MVP-163：全部模块最终仍把分离区域外接成一个`FrameRect`。远离的text-focus、status、center、floating source/target、old/new hover frame会把中间未变化像素一并paint/upload。Close prompt overlay与dialog、跨floating tab drop以及全部floating-window header damage尤其容易接近整窗。

PERF-MVP-173：floating window、host-page tab/template node和tab-drag group lookup继续用`row_data`，每个candidate可深clone完整wide DTO；damage发现不应在route之后第二次复制或扫描同一generation模型。

PERF-MVP-176：pane、chrome、host-page、toolbar damage预先假设callback可能改变整个center band/status/sibling pane，因而即使实际只改变一个control也请求大区域和frame update。最终typed dispatch result必须由mutation owner报告精确dirty domains/frames，再与focus/capture边沿合并；不能只把这些保守函数换个位置继续调用。

## 动态验收

构造相距较远的两处hover/focus更新、多个floating窗口、same/cross-group tab drop、passive/handled pane click和toolbar action，记录changed area、requested area、painted/uploaded pixels、candidate clone/visited、frame update与full-frame原因。固定容量region set应保留分离区域；未变callback damage=0；route/damage共享generation index；full-frame只能由明确阈值或未知结构mutation触发并计数。保持z-order重叠、tab relocation、status/center更新、focus、hover、clip和pixel parity。
