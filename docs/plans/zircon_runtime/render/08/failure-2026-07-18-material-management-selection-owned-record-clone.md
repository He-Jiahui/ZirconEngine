---
handoff_kind: failure
status: open
created_at: 2026-07-18
summary_slug: material-management-selection-owned-record-clone
origin_plan: docs/plans/zircon_runtime/render/03-gpu-scene-gpu-driven.md
fixing_plan: docs/plans/zircon_runtime/render/08-material-shader-permutation.md
origin_child_dir: docs/plans/zircon_runtime/render/03
fixing_child_dir: docs/plans/zircon_runtime/render/08
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/core/framework/render/material/management/selection.rs
tests:
  - cargo +1.94.1 test -p zircon_runtime --lib graphics::tests::render_product_advanced::gpu_driven_product::render_product_gpu_scene_multi_draw_64_instances_matches_cpu_fallback --locked --jobs 1 --color never -- --exact --nocapture --test-threads=1
---

# Render08: material management selection 必须克隆 owned record

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/render/03-gpu-scene-gpu-driven.md`
- 来源执行切片：GS-M4 64-instance multi-draw product evidence gate
- 修复责任计划：`docs/plans/zircon_runtime/render/08-material-shader-permutation.md`
- 交接原因：最低共享错误位于 Render08 的 framework material management selection，不属于 Render03 GPUScene、indirect submission 或产品证据 scope。

## 失败现象与复现证据

- Render03 managed GPU reservation `fd933159ee9d4f5a993721b85c576450`、job `677e6af32cfb4a7b8ab90159712cb6a5`、run `e77e3dad12c840008de054cbaa2e9792` 在进入 exact 产品测试前 terminal/released exit 101，无 live PID。
- Rust 1.94.1 编译 `zircon_runtime` lib test 时在 `selection.rs:51/52/53/59` 报告 4 个 `E0308`：`Vec<&RenderMaterialManagementRecord>` 被传给要求 `&[RenderMaterialManagementRecord]` 的 summary/status/issue 构造器，并被写入要求 owned record 的结果字段。
- broken source 在 `records_by_id.get(material_id)` 返回 `&&RenderMaterialManagementRecord` 后执行 `record.clone()`，只克隆引用而不是 record。
- 编译结束后共享工作树已出现 `selected_records.push((**record).clone())` 候选改动，但该路径当时无 coordinator lease/attribution，且没有 fresh managed compile、review 或 commit，因此不能把失败结果改写成通过。
- RenderDoc 和 ignored exporter 均未执行；`plan03_gpu_scene_multi_draw_64_instances_wgpu_20260718.png` 与配对 RDC 均未生成。

## 最低共享层根因

Material selection 的索引有意借用输入 records，但 selection 结果的公共合同拥有完整 records。broken implementation 对 `HashMap<ResourceId, &RenderMaterialManagementRecord>::get` 返回的二次引用使用浅层 `clone`，让借用类型泄漏进 owned 输出构造链。

## 架构修复验收

- Render08 owner 以显式 owned record clone 保持 request order、duplicate-id collapse、missing-id 和 summary/status/issue 一致性，并增加或确认 selection focused tests。
- 在 immutable current source 上通过 material management focused gate，以及本记录 frontmatter 中的 Render03 原始 exact compile/product reproduction。
- Render03 owner 获得 fixed return 后重新运行 DX12 WGPU parity + ignored PNG exporter + RenderDoc capture，PNG/RDC 两个 exact artifact 同时存在才可关闭 GS-M4 evidence slice。

## 禁止临时方案

- 不得把 selection 的 owned `records` 公共合同改为借用引用来绕过编译错误。
- 不得跳过 summary/status/issue 构建、弱化 Render03 产品断言或以旧 test binary/capture 代替 current-source upward gate。
- 不得由 Render03 会话吸收 material management 源路径或未归属的共享工作树改动。

## 修复结果与回传

Open state: `current-source ownership repair present; managed validation pending`.

- The selection index still borrows its input records, but `RenderMaterialManagementSelection::from_records` explicitly clones `(**record)` into the public owned result before deriving summary, status, and issue indexes.
- Request ordering, duplicate-id collapse, and missing-id behavior remain in the same selection owner; no borrowed-record compatibility path was introduced.
- The required managed material-management and Render03 product gates, including PNG/RDC evidence, remain outstanding. This handoff therefore stays `open` and does not return `fixed`.
