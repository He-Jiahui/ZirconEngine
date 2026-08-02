---
handoff_kind: fixed
status: fixed
created_at: 2026-07-15
summary_slug: text-physical-owner-hard-cut-compile-break
origin_plan: docs/plans/zircon_runtime/text/03-line-breaking-measure-and-layout.md
fixing_plan: docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md
origin_child_dir: docs/plans/zircon_runtime/text/03
fixing_child_dir: docs/plans/zircon_runtime/frameworks/05
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/text
  - zircon_runtime/src/core/framework/text
  - zircon_runtime/src/core/framework/render/mod.rs
  - zircon_runtime/src/lib.rs
tests:
  - cargo test -p zircon_runtime --test runtime_text_multilingual_product_framebuffer --locked export_runtime_multilingual_text_product_framebuffer_png -- --exact --ignored --test-threads=1 --nocapture
resolved_at: 2026-07-15
---


# Frameworks05：文本物理所有者硬切中间态破坏默认特性编译

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/text/03-line-breaking-measure-and-layout.md`
- 来源执行切片：Text03 M4 真实 GPU framebuffer 导出验收
- 修复责任计划：`docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md`
- 交接原因：最低共享原因是 Frameworks05 M3 正在执行的文本物理所有者硬切；该切片负责把 `graphics/text` 与 `core/framework/render/text` 收束到 `zircon_runtime/src/text`，Text03 不应在旧路径上补兼容层或越权完成半套迁移。

## 失败现象与复现证据

2026-07-15 的 Text03 M4 GPU 受管作业 `f4bb789f61064cccb0e9f4e2731d64bb` 在进入渲染前失败，退出码为 `101`。同一默认特性测试二进制此前已完成新增 sRGB 断言单测 `1 passed / 0 failed`；随后共享工作树出现 `zircon_runtime/src/text` 新所有者、旧 `zircon_runtime/src/graphics/text` 与 `zircon_runtime/src/core/framework/render/text` 删除，但新文件仍保留旧 `crate::graphics::text` / `crate::core::framework::render` 文本引用，导致约 200 个 `E0432` / `E0433` / `E0425` 编译错误。

首个共同错误为 `zircon_runtime/src/text/shaping/horizontal/projection.rs` 的可见性路径仍指向 `crate::graphics::text::shaping`；随后 atlas、font、cache、layout、raster、shaping 等新所有者继续引用已删除的旧模块。受影响命令为：

```powershell
cargo test -p zircon_runtime --test runtime_text_multilingual_product_framebuffer --locked export_runtime_multilingual_text_product_framebuffer_png -- --exact --ignored --test-threads=1 --nocapture
```

预期结果是默认特性编译通过、实际执行 1 条 ignored GPU exporter 并把真实 framebuffer PNG 仅写入 `docs/tests/runtime/text`；当前结果是在 GPU 初始化前编译失败，未生成或接受截图。

2026-07-15 13:27 后复扫确认 `zircon_runtime/src/text` 内部旧 owner 引用已降为 0，但仍有 5 个 renderer 消费点从 `core::framework::render` 导入已迁移的字体/字形类型：

- `zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_atlas/tests/mod.rs`
- `zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_font_bake/distance_field.rs`
- `zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_font_bake/offline_source.rs`
- `zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render/vertices.rs`
- `zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render/tests/mod.rs`

这 5 个当前残留仍由同一 Frameworks05 硬切 scope 持有；Text03 不在 renderer 调用点局部引入旧路径 shim。

## 最低共享层根因

文本物理目录已先完成移动/删除，模块可见性、内部绝对路径、公共导出和上层消费方尚未在同一硬切原子范围内完成。旧所有者已消失而新所有者仍依赖旧命名空间，因此任何默认特性文本消费者都会在模块解析阶段失败；这不是 Text03 framebuffer 断言或渲染算法故障。

## 架构修复验收

- Frameworks05 M3 完成单一物理所有者硬切：`zircon_runtime/src/text` 内部不再引用已删除的 `crate::graphics::text` 或 `crate::core::framework::render::text` 所有者。
- 默认特性 `zircon_runtime` 文本集成测试完成编译，且不通过旧路径重导出、兼容模块或别名恢复旧所有者。
- 原始 Text03 M4 GPU exporter 实际执行 `1 passed / 0 failed`，输出真实 framebuffer PNG 到 `docs/tests/runtime/text`，并确认受管 `target` 中不存在同名截图副本。

## 禁止临时方案

- 不得恢复 `graphics::text` / `render::text` 兼容模块、`pub use` 旧路径、静默 fallback、重复所有权或调用点例外。
- 不得削弱 GPU exporter、颜色覆盖、编辑几何或截图路径验收来隐藏编译失败。
- 不得把纯文本策略页或人工图片替代真实 framebuffer 证据。

## 修复结果与回传

- 根因：Frameworks05 M3 moved and deleted the old graphics/render text directories before all module wiring and consumer imports were changed atomically, leaving current-source references to owners that no longer existed.
- 架构修复：Completed the no-compatibility hard cut to the single zircon_runtime::text implementation owner and neutral core::framework::text contract, migrated all production consumers, removed both old directories and facade exports, and moved graphics module identity to the neutral render framework owner so UI no longer depends on graphics.
- 验证：Frameworks05 boundary guard 7/7 and companion boundary suites 27/27 passed; fresh Windows managed Runtime build job 1ab4722f49a04e8a8508a46333ef81e2 exited 0; original Text03 GPU exporter job 5039b9c015114ebb9b03f1fcc009ac81 exited 0 with 1/1, produced the sole 1080x2000 PNG at docs/tests/runtime/text/runtime_text_mixed_bidi_source_geometry_product_framebuffer_20260715.png, SHA256 30793B75AC50FD95B558DAFE5B8B98C9DB6C67737FF9B409FF6CCEFE53384D42, and no same-named artifact exists under the managed target.
- 回传：Text03 may resume its M4 acceptance from the fixed artifact; the physical owner migration and default-feature compile/GPU blocker are resolved without restoring any legacy path.
