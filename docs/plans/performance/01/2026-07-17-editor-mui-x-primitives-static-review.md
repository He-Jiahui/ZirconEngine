---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/mui_x_primitives.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/mui_x_primitives
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/06-component-library-mui.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - docs/plans/zircon_runtime/render/13-texture-pipeline.md
reference_sources:
  - dev/slint/internal/core/item_rendering.rs
  - dev/bevy/crates/bevy_ui_render/src/lib.rs
tests:
  - MUI X color/state/raster geometry tests
  - current-source Windows Cargo pending
  - 1/100/10000 component raster/theme-lock/command trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor MUI X primitives逐文件性能静态审查（2026-07-17）

## 范围与覆盖

`mui_x_primitives.rs`与`mui_x_primitives/` directory tree 共 **53/53** 个Rust文件、**2,050** 行已逐文件阅读。覆盖TreeView、DataGrid、date/time pickers、line/bar/pie/sparkline/gauge charts、agent chat/composer及shared dispatch/tests。当前源Cargo、产品trace、规模计数与像素验收未完成，因此仍留在`pending.md`。

## P0：图表在paint线程逐帧软件栅格

Line、pie、sparkline与gauge每次paint都按plot尺寸新建`vec![0; width * height * 4]`，上限192x192，即单图单帧约147 KiB清零分配；随后用手写像素循环计算segment distance、disc、arc的`sqrt/atan2`或pie角度并写RGBA。结果作为owned bytes进入image command，resource key只为`mui-x-chart:{kind}:{width}x{height}`。

该key不包含theme generation、gauge value、pie selected/checked hole state或node/data generation。即使当前consumer按bytes重传而没有立刻产生陈旧纹理，key也不能作为未来Render13 resource identity；一旦按key复用就会把不同值/主题错误合并。PERF-MVP-187要求优先用RHI可批处理的typed line/arc/pie/gauge geometry；若Softbuffer或复杂图表需要raster，必须由有界worker/cache按`(kind,size,data/state generation,theme generation)`构建一次，paint只读handle。Stable frame RGBA allocation/raster/upload为0。

## P0：shared quad无条件主题锁

`shared::push_quad`无论`border_width`是否为0，都会先调用`current_host_palette()`再计算可选border。几乎所有MUI X几何都经此入口，所以每个quad额外取得一次全局`RwLock`；调用者又为surface/colors独立取palette。静态路径量化：DataGrid约7次palette读取，picker field+popup最多约8次，三行TreeView的surface、row、marker可约14次。Charts/chat也按surface、plot、mark及每quad重复读取。

PERF-MVP-188的最小修复先让0-width quad不访问palette，并让每个component入口传入一次theme snapshot；最终EditorUI08按component generation编译commands，stable theme/geometry build为0。它是PERF-MVP-182/184在MUI X consumer上的明确验收切片，不允许保留consumer私有主题cache。

## 动态验收

在1/100/10,000个TreeView/DataGrid/picker/chat/chart组件与64/128/192像素图表上记录theme locks、RGBA allocations/bytes、raster pixels、sqrt/atan2 calls、host/RHI commands、uploads及CPU p95。Changed component theme acquisition<=1，zero-border quad额外theme read=0；stable generation所有build/raster/lock/upload计数为0。Chart resource identity必须区分theme、gauge value、pie hole和data generation，cache entries/bytes有界并支持device loss。保持组件kind/state/layout、chart pixels、clip/z/opacity及GPU/Softbuffer parity。
