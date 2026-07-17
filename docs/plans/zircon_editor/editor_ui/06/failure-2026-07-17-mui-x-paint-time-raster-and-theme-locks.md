---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: mui-x-paint-time-raster-and-theme-locks
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor_ui/06-component-library-mui.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor_ui/06
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/mui_x_primitives
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/mui_x_primitives.rs
tests:
  - MUI X chart generation/cache identity
  - MUI X per-component theme acquisition count
  - GPU and Softbuffer component pixel parity
---

# EditorUI06：MUI X paint-time raster与主题锁扇出

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：F4 MUI X primitives 53/53 Rust文件逐文件静态审查
- 修复责任计划：`docs/plans/zircon_editor/editor_ui/06-component-library-mui.md`
- 共同责任：`docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`、`docs/plans/zircon_runtime/render/13-texture-pipeline.md`
- 交接原因：EditorUI06拥有MUI/X组件schema与typed data generation；EditorUI08和Render13只能消费其稳定identity，不能在paint或纹理层反推组件状态。

## 失败现象与复现证据

PERF-MVP-187确认line/pie/sparkline/gauge在每次paint新建最大192x192 RGBA并用手写像素循环执行line distance、disc、arc和pie角度栅格。单图单帧最多约147 KiB清零分配，结果以owned bytes进入command。Resource key只有kind与尺寸，不含theme、gauge value、pie selected/checked或data generation，不能成为正确的资源缓存identity。

PERF-MVP-188确认MUI X shared `push_quad`即使border width为0也先读取全局palette锁；调用者又单独获取颜色palette。DataGrid单组件约7次、picker field+popup约8次、三行TreeView约14次palette读取。Stable component geometry仍每帧重建全部quad。

## 最低共享层根因

MUI X schema/component generation没有发布typed primitive/data identity，paint consumer同时承担组件分类、主题投影、几何构造、软件栅格和纹理payload所有权。Shared helper又隐藏全局同步读取，使调用者无法证明一次component只消费一个theme generation。

## 架构修复验收

- EditorUI06定义typed chart/component data与state generation；value、selection、theme和size变化进入明确cache key，stable component不重建schema/geometry。
- 优先向Render13提交typed line/arc/pie/gauge geometry；必须raster时由有界worker/cache构建并只返回handle，paint线程RGBA allocation/raster=0。
- 每changed component只借用一次EditorUI08 theme snapshot；zero-border quad不得访问palette，stable generation theme lock与command build=0。
- 1/100/10,000 components和64/128/192尺寸记录locks、raster pixels/bytes、sqrt/atan2、commands、uploads与p95；cache entries/bytes有硬上限。
- TreeView/DataGrid/picker/chat/chart的kind、state、geometry、theme切换、clip/z/opacity及GPU/Softbuffer pixels等价。

## 禁止临时方案

- 不得只把每帧生成的RGBA塞进无界`HashMap<String, Vec<u8>>`。
- 不得继续使用不含value/theme/data generation的chart resource key。
- 不得让每个MUI X component持有独立主题authority或按paint轮询主题锁。

## 修复结果与回传

Open state: `待 EditorUI06 + EditorUI08 + Render13 建立typed component/theme/resource generation并回传Cargo、规模trace和pixels parity`。
