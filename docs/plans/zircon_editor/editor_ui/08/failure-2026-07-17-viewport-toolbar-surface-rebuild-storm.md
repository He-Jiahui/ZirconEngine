---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: viewport-toolbar-surface-rebuild-storm
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor_ui/08
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute/viewport_surfaces.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute_viewport.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/native_window_presenters.rs
  - zircon_editor/src/ui/retained_host/app/viewport_toolbar_projection/surface_frames.rs
  - zircon_editor/src/ui/retained_host/app/viewport_toolbar_projection/surface_frames/docked.rs
  - zircon_editor/src/ui/retained_host/app/viewport_toolbar_projection/surface_frames/floating.rs
  - zircon_editor/src/ui/retained_host/app/viewport_toolbar_projection/surface_frames/pane_frame.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/viewport_toolbar/bridge.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/presentation/snapshot.rs
  - zircon_editor/src/ui/retained_host/viewport_toolbar_pointer/handle_click.rs
  - zircon_editor/src/ui/retained_host/viewport_toolbar_pointer/sync_surface_frame.rs
  - zircon_editor/src/ui/retained_host/viewport_toolbar_pointer/rebuild_surface.rs
tests:
  - unchanged slow-recompute toolbar build-count test
  - multi-pane toolbar cache hit/miss test
  - resize/projection/route generation invalidation test
---

# Editor UI 08：slow recompute 重建全部 viewport toolbar surface/frame

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：F4 retained-host slow recompute、toolbar projection 与 presentation snapshot 静态审查
- 修复责任计划：`docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`
- 共同责任：`docs/plans/zircon_editor/editor_ui/02-layout-taffy-and-containers.md`、`docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md`
- 交接原因：缓存 key/失效与 workbench shell/presentation authority 跨层，不能在单个 pane helper 中塞一个 stale frame cache。

## 失败现象与复现证据

每次非 paint-only host recompute 都调用 `attach_viewport_toolbar_surface_frames_to_ui`。它先深 clone
`HostWindowPresentationData`，再处理四个 docks 与所有 floating windows。每个 Scene/Game pane 都
无条件重跑共享 toolbar bridge 的 layout/hit/render projection，并新建临时 `UiSurface`/control tree、
执行 full rebuild，最后通过 `surface_frame()` 深 clone arranged tree/render extract/hit grid/report。
尺寸、toolbar projection 与 hit-control mapping 未变化时也没有 cache hit 路径。

451-file retained app 审查进一步确认，toolbar click 为取得一个 surface frame 会 `get_host_presentation()` 深 clone 全图；slow recompute 的 world-space viewport sync 与 native-window presenter 也重复消费/应用整份 presentation。viewport size 改变时先 dispatch resize，随后在同一 recompute 再 build chrome/model，放大 toolbar/pane/native-window projection。此次审查未在 helper 层加入容易陈旧的 frame cache。

31-file viewport-toolbar pointer 审查还发现 click callback 曾把该 surface 已提交的全部 controls 替换成当前单个 control，再 full rebuild；连续 A→B 点击后 A 会暂时从 hit tree 消失。本轮已直接改为 action-key upsert、same-frame no-op，并保留其他 controls。该局部修复不替代本计划的 generation cache：`sync_surface_frame()` 仍扫描/复制完整 arranged controls，变化时仍重建所有 surface routes。

2026-07-30 app viewport adapter current-source 8/8进一步确认上游click仍很宽：每次先对相同surface size重跑toolbar layout并忽略返回值，再同步pointer bridge；随后`get_host_presentation()`深clone完整presentation，只为扫描4个dock与floating windows取得一个`UiSurfaceFrame`。fallback owned-clone全部view instances后线性查找，layout frames在同一次click可读取两次。该成本未被same-frame pointer no-op消除，继续归本failure的projection/size/route generation cache。

同日viewport-toolbar projection current-source 6/6把slow-path成本量化到具体owner：入口clone/replace整presentation；4 dock在pane-kind判断前分配surface-key String；floating逐row取得owned window并重建完整ModelRc。每个有效pane调用无cache的`recompute_layout`重建template surface/host projection，再新建UiSurface、逐control构造path/metadata/route Strings、full rebuild并snapshot。`pane.viewport.clone()`还会深clone旧toolbar frame后立即覆盖。相同size/generation跨pane必须共享immutable frame，不能让单mutable bridge顺序重复构建；见`../../../performance/01/2026-07-30-editor-retained-viewport-toolbar-projection-current-review.md`。

## 最低共享层根因

toolbar 的 compiled projection、尺寸布局、pane hit mapping 与拥有所有权的 frame snapshot 没有显式
generation/cache key；presentation 更新也只有“clone full snapshot → replace full state”，没有 scoped
patch authority。

## 架构修复验收

- 以 `{toolbar projection generation, UiSize, hit-control mapping/version}` 缓存不可变 frame；同 key
  跨 dock/floating pane 复用，key 变化精确失效。
- host presentation 在内部 authority 下 patch toolbar field；不要为单字段变化深 clone/replace 全图。
- unchanged slow recompute 的 toolbar layout、temporary surface、frame deep clone、presentation full clone
  计数为 0；1/4/16 panes 给出 p50/p95 与 allocation bytes。
- resize、toolbar enable/state、route remap、floating close/reopen、device/window rebuild 均得到新 frame，
  pointer hit 与 paint projection 保持一致。
- toolbar click、world-space sync 与 unchanged native window 不得 deep clone/apply 完整 presentation；按 window/surface generation patch，resize 同帧 chrome/model build=1。
- 同 action/frame 的 1k click pointer rebuild=0；连续 A→B 后 A 仍命中；完整 surface frame 只在 toolbar projection/size/hit mapping generation 变化时重新扫描与提交。
- 1k same-size/same-generation click的toolbar layout、full presentation clone、view-instance clone与重复layout-frame read全部为0；click只按stable surface/frame handle定位，不扫描4 dock/floating全图。
- stable slow recompute的dock key/floating row/viewport old-frame clone、template layout/host projection、surface/path/metadata/route String/frame snapshot均为0；同key跨pane build=1并共享handle，per-window只patchchanged generation。

## 禁止临时方案

- 不得仅按 width 缓存；projection/route/hit mapping 变化会得到 stale input surface。
- 不得把 full rebuild 放到 worker 后无限排队，或共享可变 `UiSurface` 跨线程。

## 修复结果与回传

Open state: `待 Editor UI 08 冻结 toolbar frame cache key 并收束 presentation patch authority`。
