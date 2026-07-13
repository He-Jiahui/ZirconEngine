---
handoff_kind: fixed
status: fixed
created_at: 2026-07-12
summary_slug: planar-filter-test-surface-export
origin_plan: docs/plans/zircon_runtime/frameworks/03-optional-features-and-profile-matrix.md
fixing_plan: docs/plans/zircon_runtime/render/18-advanced-lighting-features.md
origin_child_dir: docs/plans/zircon_runtime/frameworks/03
fixing_child_dir: docs/plans/zircon_runtime/render/18
related_code:
  - zircon_runtime/src/graphics/tests/render_product_planar_reflection.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/planar_filter/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/mod.rs
tests:
  - cargo test -p zircon_runtime --lib --locked --offline --jobs 1 --no-run
resolved_at: 2026-07-13
---


# Render 18：Planar filter 产品测试引用未导出的内部常量

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/frameworks/03-optional-features-and-profile-matrix.md`
- 来源执行切片：Frameworks 03 M1 默认 feature Runtime 全量回归
- 修复责任计划：`docs/plans/zircon_runtime/render/18-advanced-lighting-features.md`
- 交接原因：失败位于 Render 18 新增的 planar-reflection 产品测试与其 `planar_filter` owner surface 之间，Frameworks 03 只负责运行包级门，不越界决定 Render 内部测试 API。

## 失败现象与复现证据

2026-07-12 15:17 以后在 Windows retained target 上按当前源码执行：

`cargo test -p zircon_runtime --lib --locked --offline --jobs 1 --no-run`

首轮编译以 E0432 失败：`graphics/tests/render_product_planar_reflection.rs` 从 `crate::graphics` 导入 `PLANAR_FILTER_PIPELINE_LABEL` 与 `PLANAR_FILTER_WORKGROUP_SIZE`，但 `advanced_lighting/mod.rs` 只向上声明 `PLANAR_FILTER_EXECUTOR_ID` 与 `PLANAR_REFLECTION_TEXTURE_RESOURCE`。修复为在 `scene_renderer -> advanced_lighting -> planar_filter` 同域链只做 `pub(crate)` 投影，测试从 `crate::graphics::scene` 读取，不扩大 `crate::graphics` 公共根。第二轮编译继续准确暴露 `RenderGraphComputeWorkload::viewport` 要求 `[u32; 3]`、planar owner 却声明 `[u32; 2]`；现已把唯一 workgroup owner 硬切为 `[8, 8, 1]`，executor 与 graph descriptor 共同消费，不在测试复制常量。第三轮 no-run 10m55s 完成，556 条均为当前工作区既有 warning，生成 test binary 晚于全部 Runtime 源码。

编译门修复后，精确执行 `render_product_planar_reflection`：3 项中 `1 passed / 1 failed / 1 ignored`。失败 `render_product_planar_reflection_changes_mirror_floor_through_camera_loop` 的 baseline 与启用 probe 输出完全相同，`changed_pixel_count=0`、mean/max error 均为 0；ignored 产品导出也生成左右完全一致的 PNG，报告的 terminal graph 不含 capture pass。底层 `render_planar` 8/8 与独立 WGPU planar-filter 3/3 均通过，故当前剩余根因位于 camera capture/filter 结果进入 main-camera material composite 的集成链，而非矩阵、filter kernel 或编译 surface。默认 feature Runtime 全量 suite 可以启动，但该产品行为门仍为 RED。

## 最低共享层根因

产品测试把 Render 18 的内部 pipeline label/workgroup 契约当作 `crate::graphics` 根 surface 消费，但模块导出链没有声明这两个测试所需名称。修复应落在 planar-filter owner 与同域测试的边界，不应把测试常量扩张成引擎公共 API。

## 架构修复验收

- 产品测试从最窄 Render 18 owner 读取所需常量，或由 owner 提供专用的 crate-private test contract；不得要求外部 crate 公共 API。（已满足）
- `cargo test -p zircon_runtime --lib --locked --offline --jobs 1 --no-run` 在修复后的当前源码上完成。（已满足）
- 精确执行 planar-reflection 产品测试；其 WGPU 环境前置若不可用，必须如实记录，不得改成静态假通过。
- 启用 probe 的产品画面必须真实包含 capture 结果，不能以只执行 camera-loop/filter 单元门代替最终材质合成；baseline 与 planar 产品输出必须产生计划阈值以上的可见差异。
- Frameworks 03 重新执行默认 feature Runtime 全量 lib suite，并以测试框架汇总更新其编号产出归档。

## 禁止临时方案

- 禁止在 `graphics` 根公开仅供内部产品测试使用的稳定公共 API。
- 禁止复制 label/workgroup magic constants 到测试文件，禁止弱化断言、删除测试或添加兼容别名。
- 禁止把本次 E0432 记为 Runtime 全量门通过。

## 修复结果与回传

- 根因：Planar capture submissions were tracked only through optional camera-order metadata, so a valid capture/main-camera loop was rejected and the product composite path remained unverified.
- 架构修复：Added an explicit camera-loop submission count owned by RenderStats, kept it separate from sorting diagnostics, and retained crate-private planar filter constants with the owner workgroup shape [8,8,1].
- 验证：Windows current-source planar product behavior 2 passed, 0 failed, 1 ignored; explicit ignored exporter passed and produced the mirror-floor PNG/report with baseline and planar submission counts 1/2.
- 回传：Returned fixed planar capture/filter/composite integration to Frameworks 03 after current-source WGPU product validation.
