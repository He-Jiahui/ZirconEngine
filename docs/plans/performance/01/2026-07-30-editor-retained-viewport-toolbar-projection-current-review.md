---
related_code:
  - zircon_editor/src/ui/retained_host/app/viewport_toolbar_projection.rs
  - zircon_editor/src/ui/retained_host/app/viewport_toolbar_projection
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/viewport_toolbar/bridge.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute/viewport_surfaces.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - docs/plans/zircon_editor/editor_ui/08/failure-2026-07-17-viewport-toolbar-surface-rebuild-storm.md
tests:
  - inline tests: 0
  - rustfmt check: blocked by pre-existing import-order drift in 4 externally modified files
  - current-source managed Windows Cargo pending
  - toolbar generation/build/clone counter and F4 pixel trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor retained viewport-toolbar projection当前源码复核（2026-07-30）

## 范围

`zircon_editor/src/ui/retained_host/app/viewport_toolbar_projection.rs`与`viewport_toolbar_projection/**`当前源 **6/6** 个Rust文件、**257** 行、**0** 条`#[test]`已逐文件阅读；path+raw-content SHA-256为`a257533e84d052443c26c04724373ffc4d3593a7b07d1322363753233c309573`。其中4个surface-frame文件含外部未提交内容，本轮只读纳入current-source审查，未修改Rust。

| 模块 | 文件 | 行 | 测试 | 当前边界 |
|---|---:|---:|---:|---|
| root/hit mapping | 2/2 | 68 | 0 | projection action到stable hit control id映射 |
| presentation/docked | 2/2 | 109 | 0 | full presentation入口与4 dock投影 |
| floating/pane | 2/2 | 80 | 0 | floating ModelRc重建与per-pane frame生成 |

## 发现

- **正向边界**：只有`Scene|Game`且`show_toolbar`的pane生成frame；其他pane清空旧frame。hit action映射为固定match，不扫描command registry；错误layout会清frame而不是发布stale geometry。
- **PERF-MVP-113 / full presentation事务**：每次调用先`get_host_presentation()`深clone完整`HostWindowPresentationData`，结束时整份`set_host_presentation`。即使没有Scene/Game pane或toolbar generation未变，仍复制/replace完整结构与嵌入payload；这与PERF-MVP-147宽snapshot叠加。
- docked路径在检查pane kind前，无条件把document/left/right/bottom四个`surface_key`各转成新String并读取width。非viewport dock仍产生String后才被helper拒绝。
- floating路径逐row调用`row_data`取得owned window，收集新Vec并重建整个`ModelRc`；每个floating window都被复制/替换，即使active pane不是viewport或frame完全相等。work随总window数增长，没有per-window generation cursor。
- **PERF-MVP-113 / 每pane重复template rebuild**：每个有效pane调用同一个mutable toolbar bridge的`recompute_layout(toolbar_size)`，它无same-size/generation guard，完整rebuild template `UiSurface`并重新投影host nodes。随后`surface_frame_for_projection_controls`再新建一个UiSurface，为每个interactive control格式化tree/path/control strings、构造metadata BTreeMap、full rebuild并snapshot。P个viewport pane为O(P×controls)构造，同宽pane也不复用。
- `let viewport = pane.viewport.clone()`会连同旧`toolbar_surface_frame: Option<UiSurfaceFrame>`一起深clone，只为closure读取tool/space/projection/orientation等少数字段；新frame随后立即覆盖clone中的旧frame。稳定slow path因此至少多复制一次旧arranged tree/render extract/hit grid。最小机械止损可在赋值前借用这些状态字段，最终仍需generation-owned frame cache；本轮因设计批准未完成未改Rust。
- `viewport_toolbar_hit_control_id`为每个interactive control产生owned String，已知静态action仍逐frame分配。最终frame应复用generation-owned typed/stable route ids；未知projection id的fallback仍需保持原文本语义。

## 参考与目标

- Slint `dev/slint/internal/core/model/repeater.rs:447-522,601-607`按changed row标脏并以generation更新实例，不为稳定consumer重建全部owned projection。
- Godot `dev/godot/scene/main/canvas_item.cpp:143-180,540-551`合并重复redraw并只在实际draw后清pending标记。Zircon可以保留多pane toolbar语义，但stable structural generation不应被每次slow recompute重新物化。

EditorUI08以`{toolbar projection generation, UiSize, hit-route generation}`缓存immutable `Arc<UiSurfaceFrame>`；相同key跨dock/floating pane共享，pane只持轻量handle。presentation用scoped toolbar patch，不先clone整树；floating store按window applied generation更新changed rows。bridge只在key变化时layout/project/build一次，hit routes使用stable ids，旧generation结果不得覆盖新frame。

## 动态验收

按panes/windows `0/1/4/16/1K`、toolbar widths `same/mixed/resize`、controls `1/16/100`、generation `stable/1% changed/full reload`与slow invalidation `1/1K/100K`记录full presentation clone/replace bytes、dock key Strings、floating row_data/ModelRc rows、viewport/old-frame clone bytes、template layout/host projection/surface rebuild/frame snapshot、route String alloc、UI p50/p95与RSS。

验收要求：stable generation的presentation clone/replace、dock/floating row visit、viewport/old-frame clone、layout/projection/surface/frame build和route String alloc均为0；同key跨pane build=1且共享handle；1%变化只patch受影响pane/window；resize/route/template generation精确失效；Scene/Game/show-toolbar、dock/floating、hit route和F4 pixels等价。managed Cargo、规模counter、F4/RenderDoc parity与independent review完成前保留在`pending.md`，不进入`review.md`。
