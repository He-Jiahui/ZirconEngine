---
related_code:
  - zircon_runtime/src/ui/tests/taffy_bridge.rs
  - zircon_runtime/src/ui/tests/taffy_layout_diagnostics.rs
  - zircon_runtime/src/ui/tests/taffy_visual_verification.rs
  - zircon_runtime/src/ui/layout/taffy_bridge
  - tools/ui-profile-capture.ps1
  - docs/zircon_runtime/ui/layout/pass.md
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/02-layout-taffy-and-containers.md
reference_sources:
  - dev/bevy/crates/bevy_ui/src/layout/ui_surface.rs
tests:
  - 6 bridge/diagnostic/document-token tests reviewed
  - transient 3-node Taffy tree build count equals one reproduced by test contract
  - visual file only checks documentation/script tokens; no product capture executed
  - current-source Cargo persistent-tree counters and RenderDoc/Softbuffer evidence pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime UI Taffy支撑测试逐文件性能静态审查（2026-07-18）

## 范围与覆盖

已完整阅读`zircon_runtime/src/ui/tests/{taffy_bridge,taffy_layout_diagnostics,taffy_visual_verification}.rs`，共3/3个tracked Rust文件、350行、6个测试。范围覆盖container→Taffy style/capability映射、fallback/non-finite contract、transient tree diagnostics以及视觉验证文档脚本token。

## PERF-MVP-261：diagnostic直接证明transient tree

`taffy_layout_report_exports_transient_tree_build_stats`在单个HorizontalBox和两个children上明确断言`taffy_tree_build_count=1`、`taffy_tree_node_count=3`。这是当前每容器临时建树的确定性基线，不是优化通过证据。persistent surface完成后，同一stable generation的第二帧及后续帧必须tree build/insert=0；changed style/children只更新affected nodes，report需区分create与compute，不能删除指标掩盖成本。

## Bridge语义门禁

4个bridge测试锁定flex/grid/wrap/block映射、Zircon-owned overlay/scroll/masonry/container fallback和non-finite拒绝。这些纯函数测试没有规模热点，但必须在persistent tree和typed style cutover后保留，防止为缓存命中扩大不受支持的Taffy ownership。

## visual token测试不是视觉验收

`taffy_visual_verification.rs`只用`read_to_string`读取`pass.md`与`ui-profile-capture.ps1`并检查字符串存在；它没有启动产品、执行interaction、生成截图、比较像素或运行RenderDoc。故该文件只能证明验证入口未被删除，不能进入`review.md`的动态/视觉证据。当前源码GPU capture、Softbuffer截图和interaction artifact仍pending。

## 验收要求

1/100 nested containers、100/1k/10k nodes稳定300帧记录tree create/insert/style/children/compute、selection entries、allocation和CPU p95；stable create/insert=0，changed work随affected subtree。另执行当前源码`ui-profile-capture.ps1`的四场景、截图像素检查和RenderDoc draw/resource/overdraw核对。Cargo与产品证据完成前，3/3留在`pending.md`。
