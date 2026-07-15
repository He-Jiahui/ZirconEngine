---
related_code:
  - zircon_runtime_interface/src/ui/surface/render/text_effects.rs
  - zircon_runtime/src/text/raster/policy.rs
  - zircon_runtime/src/text/font/decoration_metrics.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render/material.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render/vertices.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/shaders/zr_text_sdf.wgsl
plan_sources:
  - docs/plans/zircon_runtime/text/05-sdf-msdf-pipeline.md
  - docs/superpowers/specs/2026-07-13-runtime-text-sdf-effects-decoration-design.md
status: complete
---

# Runtime Text05 SM-M4 SDF 效果、装饰线与变换采样实施计划

> **For Codex:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Follow repository milestone-first policy: write tests with slices, defer Cargo execution to each named testing stage, and append one output row immediately after each accepted slice.

**Goal:** 完成 Text05 SM-M4 的 outline、shadow、MTSDF glow、face-derived underline/strikeout、material group 2 及旋转/透视 `screenPxRange`，并产出真实 WGPU 截图证据。

**Architecture:** 公共 DTO 只表达效果与装饰；raster policy 决定 bitmap/SDF/MSDF/MTSDF；font owner 解析 face 装饰度量；SDF renderer 以 material draw plan + group 2 动态 uniform 消费；shader 同时支持 CPU 快路径和 fragment-derived 通用路径。编辑态装饰、atlas ownership、layout/shaping 保持独立。

**Tech Stack:** Rust, serde, ttf-parser, bytemuck, wgpu, WGSL, image, repository session/cargo coordinator.

---

## SM4-M1 公共效果契约与距离场选路

实施切片：

1. 在 folder-backed `text_effects.rs` 增加 outline/shadow/glow/decorations DTO、默认值与归一化；从 `UiResolvedStyle` 投影到 `UiTextPaint` 和内部 text batch。
2. 将归一化效果转换为 `GlyphRasterEffects`，接通 `distance_field_mode_for_request`；小字号效果强制 SDF，glow 强制 MTSDF，颜色/subpixel bitmap 规则保持不变。
3. 增加 DTO serde/normalization、paint projection、生产选路测试；禁止使用具体字体名或测试专用分支。

测试阶段 `SM4-T1`：

- Compile：coordinator-managed `cargo check -p zircon_runtime_interface --locked --jobs 1`；`cargo check -p zircon_runtime --lib --no-default-features --features target-client --locked --jobs 1`。
- Unit：精确运行 `text_effect`、`text_policy_*effect*`、paint projection filters。
- Debug/correction：从 interface DTO -> paint -> batch -> raster policy 自下而上定位，不在 renderer 顶层覆盖错误选路。
- Acceptance：有效效果的生产 batch mode 与策略一致；无效果序列化兼容；相关 source 文件仍低于结构预算。
- Docs：更新 `docs/zircon_runtime/graphics/text/sdf.md` 和 Text05 numbered output archive。

## SM4-M2 Face 装饰度量与几何

实施切片：

1. 新建 `graphics/text/font/decoration_metrics.rs`，从 post underline / OS/2 strikeout 读取并按 units-per-em 缩放；缺表使用计划指定 em 回退。
2. 将 rich/plain underline/strikethrough 作为排版样式投影，不扩展编辑态 `UiTextPaintDecorationKind`；按 run baseline 生成矩形，V1 不做 skip-ink。
3. 删除 `rich_text.rs` 的固定 1px/run-bottom 下划线策略；添加 face table、fallback、字号缩放、underline/strikeout geometry 测试。

测试阶段 `SM4-T2`：

- Compile：coordinator-managed target-client check。
- Unit：`render_text_decoration_underline_geometry`、`render_text_decoration_metrics_from_face_tables`、strikeout/fallback/scale filters。
- Debug/correction：先验证 ttf-parser/asset metrics，再验证 baseline 坐标转换，最后验证 render draw order。
- Acceptance：有表字体不使用 em 回退；缺表路径可见且稳定；selection/caret/composition 行为不变。
- Docs：新增或更新 source-mirrored font decoration 文档及 SDF 文档。

## SM4-M3 Group 2 材质、效果 shader 与 draw plan

实施切片：

1. 新建 `sdf_render/material.rs`，实现材质语义、16-byte ABI、device uniform alignment、动态 offset 和 draw ranges；`sdf_render.rs` 保持编排薄层。
2. 扩展 `zr_text_sdf.wgsl`：fill、outline、derivative-offset shadow、MTSDF true-distance glow 与 straight-alpha over；atlas 仍为 group 0，material 固定 group 2。
3. 扩展 prepare report，记录 material/draw/effect counts；增加 WGSL parse、ABI、draw coalescing、outline/shadow/glow 数值测试。

测试阶段 `SM4-T3`：

- Compile：coordinator-managed target-client check。
- Unit：`render_text_outline_thickness_matches_distance_offset`、`render_text_shadow_offset_correct`、glow true-distance、material ABI/draw plan filters。
- Debug/correction：shader 数学先用 CPU mirror 测试，再 WGSL parse，再最小 WGPU render；禁止用截图掩盖数值错误。
- Acceptance：effect 不进入 atlas identity；每个 material 边界 draw/offset 正确；SDF/MSDF/MTSDF 无效果回归通过。
- Docs：更新 renderer/material source-mirrored 文档和 Text05 archive。

## SM4-M4 旋转/透视 screenPxRange 与真实产品证明

实施切片：

1. 增加 `CpuScreenSpace` / `FragmentDerived` 两档投影契约；fragment 路径按 atlas dimensions 和 UV derivatives 推导 pixel range，shadow 使用 `dpdx/dpdy` 保持屏幕偏移。
2. 扩展真实 WGPU product fixture，展示 fill/outline/shadow/glow、underline/strikeout、45° rotated MSDF 与 perspective-scaled MTSDF；加入像素覆盖、颜色、边缘宽度与非空 framebuffer 断言。
3. PNG 写入 `docs/tests/runtime/text/runtime_text_sdf_effects_transformed_product_framebuffer_20260713.png`；断言 repo `target` 和 coordinator target 中没有该图副本。

测试阶段 `SM4-T4`：

- Compile/build：coordinator-managed target-client check；产品测试 `--no-default-features --features target-client` 构建。
- Unit/integration：`render_text_sdf_rotated_screen_px_range_sharp`、`render_text_msdf_3d_space_sharp_at_distance`、完整产品 framebuffer exact filter。
- Visual QA：读取 PNG 尺寸/hash/color count，使用本地图片查看器按原分辨率复核；截图必须是产品输出，不得是策略文字页。
- Debug/correction：先验证 shader derivative 数值，再修投影/裁剪，最后才更新基准证据。
- Acceptance：旋转与透视边缘保持约 1px AA；效果和装饰均可见；证据只位于 `docs/tests/runtime/text`。
- Docs：更新 SDF 文档、Text05 主计划、numbered output archive 和本实施计划状态。

## SM4-M5 结构、回归与里程碑关闭

实施切片：

1. 执行 `engine-code-structure-convention` 优先审计：production file budget、folder-backed tests、module root 薄层、magic constant owner、shader `zr_` 命名与禁用兼容 shim。
2. 执行 Text05 精确回归：SM2 dynamic SDF/MSDF/MTSDF、SM3 `.zsdf` offline、SM4 effects/decorations/transformed proof；确认 target 无 PNG/`.zsdf` 残留。
3. 更新 `05-sdf-msdf-pipeline.md` 和 numbered archive 的 SM-M4 状态/证据；运行 plan-output/failure audit，只记录并隔离外部 editor 计划问题。

测试阶段 `SM4-T5`：

- Compile：coordinator-managed target-client check。
- Unit/integration：SM2+SM3+SM4 exact filters；structure priority exact filters。
- Audit：`git diff --check` scoped audit、oversized production files、conflict markers、target artifact scan、plan-output/failure audit。
- Debug/correction：任何上层失败先回到 DTO/policy/font/material/shader 的最低共享层修复，再向上重跑。
- Acceptance：每个完成切片有独立 output row；SM-M4 所有计划测试与真实产品证据通过；外部 editor 交接错误未被本 Session 越权修改。
- Docs：把本计划 `status` 改为 `complete`，Text05 SM-M4 改为完成；不提前关闭仍未完成的 Text05 后续里程碑。

## 状态与产出记录

| 里程碑/切片 | 状态 | 日期 | 证据 |
| --- | --- | --- | --- |
| SM4-M1 public effects/routing | accepted | 2026-07-13 | Runtime target-client `d0393afe...` exit 0；interface/paint tests 3/3。 |
| SM4-M2 face decorations | accepted with target-client composite gate deferred | 2026-07-13 | graphics `1e33672f...` exit 0；`text_decoration` 7/7。 |
| SM4-M3 group2 material/effect shader | accepted | 2026-07-14 | graphics `3cbfb0ec...`/`7872907f...` exit 0；M3 material/effect tests pass in `417061de...`；target-client `c87fb5aa...` exit 0。 |
| SM4-M4 transformed product | accepted | 2026-07-14 | WGPU `4daaa9cd...` and `417061de...` product 1/1；PNG `D0BD287F...649BD59`；121/122 + tolerance exact `8dc2b7e2...` 1/1 = current group 122/122。 |
| SM4-M5 structure/closeout | accepted | 2026-07-14 | `8dc2b7e2...` production/test budgets 2/2、UI child-owner split 1/1；scoped diff/naming/conflict/target hygiene passed；foreign plan-output violations isolated。 |
