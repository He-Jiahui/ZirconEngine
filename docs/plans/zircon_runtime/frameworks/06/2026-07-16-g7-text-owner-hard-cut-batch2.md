# Frameworks 06 G7 Text Owner Hard Cut Batch 2

> 本文件记录 `06-development-conventions-and-guardrails.md` 的 G7 docs 勾稽修正批次；父计划仍持有完整里程碑定义。

| 切片 | 完成项目 | 状态锚 | 日期 | 当前状态与证据 |
|---|---|---|---|---|
| M1 / G7 | Text owner machine-path, command and prose hard cut, batch 2 | `frameworks_06_g7_text_owner_batch2_accepted_global_red` | 2026-07-16 | **batch accepted / global G7 RED**。在 13 份原本 clean 的非正式计划文档中，将 84 条机器路径、历史命令和正文链接从退役 `core/framework/render/text/*`、`graphics/text/*`、root `font_sdf_build_tool`、renderer-local `ui/sdf_font_bake*` owner 硬切到唯一现存 `text/model/*`、`text/*`、`text/service.rs`、`text/font_sdf_build_tool`、`text/sdf/font_bake*` owner，并将旧 Rust test filter 硬切到 `text::sdf::font_bake::tests`；不保留 alias、shim 或旧路径解释层。`python tools/check_conventions.py --only docs --json` 在本批开始时为 1,162 documents / 66,764 paths / 347 missing / 74 affected docs；最终 fresh 运行因并发文档增加 4 个被检查路径而为 1,162 / 66,768 / 308 / 62，本批清零 39 个机器路径 violation。scoped `git diff --check` 通过，所选五类旧前缀全文扫描为零，50 个新增唯一源码目标全部存在。首轮独立 review 为 0 Critical / 2 Important / 0 Low；后续扩大语义扫描共补充修正 17 条 checker 未覆盖的旧模块名、source owner 与 font-bake 所有权描述；最终 fresh re-review 为 0 Critical / 0 Important / 0 Low。G7 仍有 308 missing、分布于 62 份文档，因此不声明 M1 或 Plan06 完成。 |

## 精确文档范围

- `docs/assets-and-rendering/runtime-ui-graphics-integration.md`
- `docs/assets-and-rendering/runtime-ui-slate-rendering-gap-audit.md`
- `docs/superpowers/plans/2026-07-12-runtime-rich-table-spans.md`
- `docs/superpowers/plans/2026-07-13-runtime-zsdf-offline-bake.md`
- `docs/superpowers/specs/2026-07-12-runtime-rich-table-spans-design.md`
- `docs/superpowers/specs/2026-07-13-runtime-zsdf-offline-bake-design.md`
- `docs/ui-and-layout/shared-ui-template-runtime.md`
- `docs/zircon_runtime/asset/assets/font.md`
- `docs/zircon_runtime/core/framework/render/common_api.md`
- `docs/zircon_runtime/graphics/scene/scene_renderer/ui/sdf_font_bake.md`
- `docs/zircon_runtime/graphics/text-cache.md`
- `docs/zircon_runtime/graphics/text/font-decoration-metrics.md`
- `docs/zircon_runtime/graphics/text/font-variation-instances.md`

## 验收边界

- 本批为 docs-only G7 support correction，不运行 Cargo，也不冒充分支 CI。
- fresh 独立 re-review 已达 0 Critical / 0 Important；通过 coordinator maintenance finalize 提交精确 14 文件 manifest。
- 后续批次继续从剩余 308 missing 中选择 clean owner；foreign dirty 文档保持原会话归属。
