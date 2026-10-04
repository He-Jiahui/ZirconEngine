---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: asset-pane-projector-repeated-model-scans
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor/09-editor-asset-management.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor/09
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/ui/layouts/views/assets_activity.rs
  - zircon_editor/src/ui/layouts/views/asset_browser.rs
  - zircon_editor/src/ui/retained_host/primitives.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/template_node_projection.rs
  - zircon_editor/src/ui/workbench/asset_content_layout/mod.rs
  - zircon_editor/src/ui/workbench/asset_content_layout/paint_metadata.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/docks/pane/template_nodes/asset_content/projector.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/docks/pane/template_nodes/asset_content/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_node_pipeline/transform.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_node_pipeline/draw.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/native_panes/scrollbar/asset.rs
tests:
  - tools/tests/test_editor09_asset_content_generation_projection.py
  - zircon_editor/src/ui/workbench/asset_content_layout/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/docks/pane/template_nodes/asset_content/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_node_pipeline_tests/transform.rs
---

# Asset pane projector repeated model scans

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：`paint_workbench_renderer/{docks/pane.rs,docks/pane/**}` 10/10 Rust文件
- 修复责任计划：`docs/plans/zircon_editor/editor/09-editor-asset-management.md`
- 交接原因：固定资产内容几何、identity index与visible range应由资产模型generation持有，painter只能执行动态scroll/hover投影。

## 失败现象与复现证据

Activity projector初始化曾两次扫描完整node model；Browser projector与Browser content scrollbar的list模式各自最坏四次扫描，之后command pipeline还会再次遍历。性能计划已把两个projector构造器与Browser scrollbar各收敛为单遍几何摘要，并删除旧search helpers；该止损仍会在每次paint重新扫描稳定模型一次。

## 最低共享层根因

资产模型generation没有同时发布content mode、panel/header/grid/preview固定几何、解析后的node identity、row counts和visible range。Painter只能从通用template node DTO重新推导结构。

## 架构修复验收

- Asset model generation change最多构建一次geometry/identity index；stable paint projector-wide `row_data`/identity parse为0。
- Scroll只更新visible range/translation，hover只更新目标row/card动态段，不重建固定几何或全量commands。
- 1/1k/10k nodes报告row_data、parse、clone bytes、alloc、CPU p95；增长由visible nodes而非total nodes主导。
- Activity/Browser list/thumbnail、header/grid/preview、empty/stale、scroll/clip/hover/hit/Softbuffer pixels等价。

## 禁止临时方案

- 不得在painter新增无generation/容量边界的第二份node cache。
- 不得通过截断nodes或取消不可见项语义来伪造常数时间。
- 不得回退为多个`find_*`从row zero重复扫描通用DTO模型。

## 修复结果与回传

Open state: `2026-07-19 generation-owned geometry/fixed+visible row plan、中立generation input、跨DTO共享元数据与旧 painter-owned identity 文件删除已落地；但 2026-08-25 重审确认 projector 仍通过 metadata 与辅助 helper 在 stable paint 重新解析 control_id，并以 contains 推断缩略图角色。因此“zero-stable-scan/zero-parse”静态结论撤回，原 static contract 存在盲点。2026-08-25 已完成按行 identity descriptor 与 row-aware transform 硬切：generation 负责 descriptor、geometry 和辅助 viewport，paint 仅按 row 消费 typed descriptor，旧 parser/index/contains 路径均删除。独立审查指出 virtual thumbnail slot 未按重绑定项更新子节点几何；现已通过 generation payload 的扩展名/类型宽度、metadata materialized card frame 与共享纯几何函数修正，且加入深滚动回归。修正后的独立复审为 0 Critical / 0 Important，格式门禁已收束。Windows 受管 focused Cargo 以 exit 101 停在既有 zr_rhi_wgpu 14 项诊断，未开始编译 zircon_editor；该 failure 仍保持 open，等待 RHI 恢复后的 Rust、像素和 1/1k/10k 动态基线后才可作性能结论。`

当前切片记录：[2026-07-19-asset-content-generation-projection.md](2026-07-19-asset-content-generation-projection.md)。重审与采样方案：[2026-08-25-asset-pane-identity-index-performance-research.md](2026-08-25-asset-pane-identity-index-performance-research.md)。

## 产出记录与时间

| 时间 | 状态 | 完成项目与当前门禁 |
|---|---|---|
| 2026-08-25 | `OPEN / source-hardcut-static-verified / focused-rust-blocked-upstream-rhi / dynamic-baseline-blocked` | 重审后完成 generation-owned dense row descriptor、descriptor 驱动 content/header/grid/preview 与辅助 viewport 几何、以及必需 `transform_row` 合同；Activity/Browser projector 不再调用 `identity`、`is_scroll_node`、reference/source-tree parser 或 thumbnail `contains`。独立审查发现的虚拟缩略图重绑定子几何漂移已由 generation payload、materialized card frame 和共享纯函数修正，并新增深滚动长文件名/双行/宽徽标回归；修正后的独立复审为 `0 Critical / 0 Important`。Python 合同 `7/7`、`rustfmt --check`、scoped diff check 通过。Windows 受管 focused Cargo 终态 exit 101，停在既有 `zr_rhi_wgpu` 14 项诊断、未编译 `zircon_editor`；外部 RHI 仍阻断真实 1/1k/10k、CPU、alloc、GPU、功耗与像素基线，不将该静态修复标记为 fixed 或性能验收。 |
| 2026-07-23 13:17 +08:00 | `OPEN / source_review_zero_validation_pending` | 独立复审 0/1/0：业务源码的 generation metadata、zero-stable-scan、visible-row plan 与旧 identity owner 删除均成立；唯一 Important 为 failure exact manifest 遗漏 `asset_content_layout/mod.rs`、asset-content `mod.rs`、template-node `transform.rs` 与 exact-row transform test。现已补齐并保留 `identity.rs` 作为待提交删除项，增量复审 0/0/0；Cargo、像素/规模/p95、fixed return 仍待。 |
| 2026-07-19 15:02-15:22 +08:00 | `OPEN / source_complete_static_green_validation_pending` | 生成期 typed metadata、中立 generation input、DTO 共享保留、Activity/Browser 零扫描投影、精确可见行与旧 parser 删除已完成，静态 6/6、反向层级依赖 0；managed Cargo、产品等价、规模数据、独立 review 与 fixed return 尚未完成。 |

## 2026-09-25 当前快照复核

本轮 successor Session `failure-roll-01a084c8-editor09-asset-pane-r2` 仅持有本 failure 文档租约；没有接管或改写现有业务源码。11 个 `related_code` 路径均存在，当前 SHA256 与工作树归属如下：

协调器 source snapshot `3809`（2026-09-25）冻结了下表 11 个源码路径；本记录正文由同一 successor 租约维护，未被纳入源码 manifest，避免记录自引用造成哈希漂移。

| 路径 | 当前 SHA256 | 工作树归属 |
|---|---|---|
| `zircon_editor/src/ui/layouts/views/assets_activity.rs` | `ae345684f3d31cac5b14335e9f03e2d72960215f9ccbbb7d3c37ede31643e484` | clean |
| `zircon_editor/src/ui/layouts/views/asset_browser.rs` | `55dd8c9cdad79d8040470b946866803148c672f587e774552cc61cc670b51451` | clean |
| `zircon_editor/src/ui/retained_host/primitives.rs` | `af6cb7b626ac17b5c72b854fd5d8a636aeac5afa2b24381bcf291237ec1ab7fc` | foreign dirty; `ModelValues<T>::Clone` implementation only |
| `zircon_editor/src/ui/retained_host/ui/pane_data_conversion/template_node_projection.rs` | `4ffccc30d802b85f7ad3ade7ed6f2149afd194ae11a7f750aa47002c1e2594ec` | clean |
| `zircon_editor/src/ui/workbench/asset_content_layout/mod.rs` | `76af986b02fa0efc877f4760977f899315af46e8f175e1b77b03cc4a4aad1327` | clean |
| `zircon_editor/src/ui/workbench/asset_content_layout/paint_metadata.rs` | `5b25032109e13c5e5d9f9393d3d0d410d2975066265c2087249cfdc8d8903e64` | foreign dirty; visible-group capacity helper/test module only |
| `zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/docks/pane/template_nodes/asset_content/projector.rs` | `80831bc90969bb3d2ef8d4e13d6689bd53877f587155bbba0b4263539795e7d6` | foreign dirty; `let-else` thumbnail slot guard only |
| `zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/docks/pane/template_nodes/asset_content/mod.rs` | `781835461ea4ce35736b23093869258a6cb18938e2f303ab5d72e4bf4540eee8` | clean |
| `zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_node_pipeline/transform.rs` | `c1af51d7204ac1dfd53a80c2abc243570c14826866f85f7d88cb2beac53901fe` | clean |
| `zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_node_pipeline/draw.rs` | `e08b7f8f1d460633d1d8e726c4aa26d68273136eade71a8fd0dcb19ad8db672a` | clean |
| `zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/native_panes/scrollbar/asset.rs` | `a2bae353b58ebb4969be8b8011ecab8359dc08b87da159e819ff642953ef46bb` | clean |

The current static evidence was rerun against this same snapshot: `python -X utf8 -m unittest tools.tests.test_editor09_asset_content_generation_projection` passed `8/8`; `rustfmt --edition 2021 --check` over all 11 Rust paths passed; and `git diff --check` over the claimed paths passed (only the repository's LF/CRLF normalization warnings were emitted). The contract test now has eight methods; the historical `7/7` entry above is retained as provenance, not reused as current evidence.

The exact Rust gates to execute after the lower `zr_rhi_wgpu` owner returns are:

- `ui::workbench::asset_content_layout::tests::generation_metadata_reports_one_identity_parse_per_input_row`
- `ui::workbench::asset_content_layout::tests::browser_generation_descriptors_own_content_and_auxiliary_viewport_geometry`
- `ui::workbench::asset_content_layout::tests::ten_thousand_browser_nodes_project_only_the_visible_thumbnail_groups`
- `ui::retained_host::host_contract::paint_workbench_renderer::docks::pane::template_nodes::asset_content::tests::projector_consumers_use_generation_metadata_without_model_scans_or_identity_parsing`
- `ui::retained_host::host_contract::paint_workbench_renderer::docks::pane::template_nodes::asset_content::tests::browser_thumbnail_projector_rebinds_bounded_slots_after_a_deep_scroll`
- `ui::retained_host::host_contract::paint_workbench_renderer::docks::pane::template_nodes::asset_content::tests::browser_thumbnail_virtual_rebind_reprojects_item_specific_child_geometry`
- `ui::retained_host::host_contract::paint_template_nodes::template_node_pipeline::tests::transform::transform_exact_row_plan_skips_unselected_model_rows`

Each filter must be run through the Windows managed `zircon_editor --lib --locked --jobs 1` wrapper with `--exact --test-threads=1`, followed by the lower RHI gate, the 1/1k/10k allocation/CPU baseline, product pixel equivalence, and upward acceptance. No dynamic test or performance result is claimed here: the last managed focused Cargo attempt exited 101 on 14 pre-existing `zr_rhi_wgpu` diagnostics before compiling `zircon_editor`, and the external `E:\Git\zr_vm` checkout is still dirty. Consequently this failure remains `open`; fixed return, review/closeout and WeCom notification are still pending.

Independent review receipt (successor snapshot): reviewer confirmed all 11 hashes, the three foreign dirty diffs, the generation-owned descriptor hard cut, and the absence of stable-paint identity/parser/thumbnail-`contains` scans. Review result: `Critical=0, Important=0, Moderate=0`. The reviewer made no source or document edits. This receipt does not waive the pending managed Cargo/RHI, product-pixel, scale/performance, fixed-return, or closeout gates.

## 2026-09-26 successor intake (failure-roll-01a084c8-editor09-asset-pane-r3)

- The stale predecessor `failure-roll-01a084c8-editor09-asset-pane-r2` owned this same failure document and had no active lease; it was cancelled through the coordinator before successor registration. Its static snapshot 3809 and the lower `zr_rhi_wgpu` admission blocker remain provenance only. Successor r3 acquired the document lease against base SHA-256 `968dc9e74569bd778305e40f0d6edb81160b0b1ab090d65fdf103bfb9ffd927e`; no business source path is leased or edited.
- The eleven related-code paths were rehashed before intake and match the current manifest: `assets_activity.rs` `ae345684f3d31cac5b14335e9f03e2d72960215f9ccbbb7d3c37ede31643e484`; `asset_browser.rs` `55dd8c9cdad79d8040470b946866803148c672f587e774552cc61cc670b51451`; retained-host `primitives.rs` `af6cb7b626ac17b5c72b854fd5d8a636aeac5afa2b24381bcf291237ec1ab7fc`; `template_node_projection.rs` `4ffccc30d802b85f7ad3ade7ed6f2149afd194ae11a7f750aa47002c1e2594ec`; `asset_content_layout/mod.rs` `76af986b02fa0efc877f4760977f899315af46e8f175e1b77b03cc4a4aad1327`; `paint_metadata.rs` `5b25032109e13c5e5d9f9393d3d0d410d2975066265c2087249cfdc8d8903e64`; asset-content `projector.rs` `80831bc90969bb3d2ef8d4e13d6689bd53877f587155bbba0b4263539795e7d6`; asset-content `mod.rs` `781835461ea4ce35736b23093869258a6cb18938e2f303ab5d72e4bf4540eee8`; pipeline `transform.rs` `c1af51d7204ac1dfd53a80c2abc243570c14826866f85f7d88cb2beac53901fe`; pipeline `draw.rs` `e08b7f8f1d460633d1d8e726c4aa26d68273136eade71a8fd0dcb19ad8db672a`; and scrollbar `asset.rs` `a2bae353b58ebb4969be8b8011ecab8359dc08b87da159e819ff642953ef46bb`. The three dirty paths (`retained_host/primitives.rs`, `asset_content_layout/paint_metadata.rs`, and asset-content `projector.rs`) retain foreign-only diffs; no source change is absorbed.
- This is a static successor intake only. The existing Python 8/8 contract check, rustfmt/diff checks, and independent C/I/M=0/0/0 review are not promoted to dynamic acceptance. Managed Windows `zircon_editor --lib --locked --jobs 1` exact filters, lower RHI repair, 1/1k/10k allocation/CPU baselines, product pixel equivalence, fixed return, coordinator closeout, and WeCom notification remain pending. The failure stays open.

### r3 successor independent review receipt

Reviewer `/root/review_editor03_gizmo_private` re-read snapshot 3936 (SHA-256 `6e38f62242d2d521df49544bc98d6b9429423f8526ad5671aac741e60bf1622c`). The stale r2 cancellation and absence of an active lease, document base hash `968dc9e74569bd778305e40f0d6edb81160b0b1ab090d65fdf103bfb9ffd927e`, all eleven related-code hashes, and exactly three retained foreign dirty paths were confirmed. The static Python 8/8, rustfmt, diff, and source-review evidence is not dynamic acceptance; managed Editor Cargo, lower RHI, pixel/scale/performance, fixed return, closeout, and WeCom gates remain pending. Independent review result: **Critical=0 / Important=0 / Moderate=0**. The failure stays open.
