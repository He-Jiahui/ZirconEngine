---
related_code:
  - zircon_runtime/src/ui/tests/surface_dirty_mui.rs
  - zircon_runtime/src/ui/surface/property_mutation/metadata_dirty.rs
  - zircon_runtime/src/ui/surface/surface/rebuild.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions/toast_timer.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/06-component-library-mui.md
  - docs/plans/zircon_editor/editor_ui/07-ui-animation-theatre.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
tests:
  - 3 dirty-domain tests reviewed
  - autoHideDuration broad invalidation reproduced by source and test contract
  - transition progress 60/120 Hz stage-counter matrix pending
  - current-source Cargo and product animation/toast trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime UI surface dirty MUI测试逐文件性能静态审查（2026-07-18）

## 范围与覆盖

已完整阅读`zircon_runtime/src/ui/tests/surface_dirty_mui.rs` 1/1个tracked Rust文件、202行、3个测试，并定向复核`metadata_dirty.rs`与surface rebuild消费。测试锁定virtual range、customization和feedback metadata的当前dirty-domain合同。

## PERF-MVP-315：属性effect过度保守并被高频放大

`metadata_attribute_dirty`把Snackbar/Alert的severity/message/icon/auto-hide duration/blur listener/anchor统一标记为layout+hit+render+text+input。测试明确要求修改`autoHideDuration`触发全部五个domain，但toast timer实际按需读取duration；计时配置不应重排整棵UI。customization的任意`slotProps/sx/classes`同样无视实际changed field而全域dirty。

transition分支更危险：`transition_progress`与`animation_progress`虽然不标layout，却总标hit+render+input；当前rebuild在`dirty.input`时全量重建arranged tree，之后hit/input/a11y等stage也被唤醒。60/120 Hz Fade/Grow/Slide/Zoom因此可能每tick支付O(N)主线程成本，即使Fade只改变opacity。

## 正确责任边界

EditorUI06应让component schema/compiled descriptor携带typed property effect：timer-only更新timer generation，text/layout/style/interaction各自精确dirty，nested slot/sx patch按changed typed fields求union。EditorUI07为每类transition声明render transform、geometry/hit或layout effect；EditorUI08按changed node/range消费domain，不因单个input flag重建全量arranged authority。不得用全组件白名单或统一render-only掩盖Grow/Slide/Zoom命中语义。

## 验收要求

对Alert/Snackbar/Modal与Fade/Grow/Slide/Zoom的1/100/10k nodes、60/120 Hz连续10k updates记录property mutations、dirty domains、layout/arranged/hit/render/input/a11y visits、timer reschedule、allocation和CPU p95。`autoHideDuration`更新pipeline visits=0且timer恰更新一次；Fade只render changed node；几何transition只patch受影响range；unchanged value dirty=0。current-source Cargo、workbench toast/transition产品trace和像素/命中对拍完成前，本文件留在`pending.md`。
