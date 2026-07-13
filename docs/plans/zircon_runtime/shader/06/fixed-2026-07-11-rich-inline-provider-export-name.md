---
handoff_kind: fixed
status: fixed
created_at: 2026-07-11
resolved_at: 2026-07-11
summary_slug: rich-inline-provider-export-name
origin_plan: docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md
fixing_plan: docs/plans/zircon_runtime/text/07-rich-text-html-bbcode.md
origin_child_dir: docs/plans/zircon_runtime/shader/06
fixing_child_dir: docs/plans/zircon_runtime/text/07
related_code:
  - zircon_runtime/src/ui/text/layout_engine.rs
  - zircon_runtime/src/ui/text/layout_engine/rich_inline.rs
tests:
  - direct rustc --test zircon_runtime/src/lib.rs using the current profiling dependency set
  - zircon_runtime_gpu_tests.exe graphics::scene::scene_renderer::environment::ibl_bake_
---


# Text 07：rich inline provider 导出名漂移

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md`
- 来源执行切片：EC-M3 fixed PMREM GPU dispatch/readback 红转绿
- 修复责任计划：`docs/plans/zircon_runtime/text/07-rich-text-html-bbcode.md`
- 交接原因：Shader06 的完整 lib 测试编译被 Text07 `layout_engine` 子模块导出边界阻断，最低共享原因属于富文本布局 owner。

## 失败现象与复现证据

当前源码直接编译 `zircon_runtime/src/lib.rs --test` 失败 1 项：`layout_engine.rs` 导出 `rich_inline::inline_rich_layout_with_provider`，但 `rich_inline.rs` 的实际实现名是 `layout_inline_rich_text_with_provider`。Shader06 GPU 聚焦测试因此无法生成包含最新断言的测试二进制。

## 最低共享层根因

`layout_engine` 根模块的 `pub(crate) use` 名称与子模块已存在实现名不一致；内部调用仍使用实际名称，说明这是导出拼写漂移，不是缺少第二套布局实现。

## 架构修复验收

- 根模块直接以 `as inline_rich_layout_with_provider` 导出现有实现，不新增重复函数或兼容模块。
- 当前源码完整 lib test 编译通过。
- Shader06 原始 39 项 `ibl_bake_` 聚焦测试重新执行并关闭 fixed-PMREM 红测。

## 禁止临时方案

- 不新增别名实现、兼容 shim、测试专用绕过或重复布局逻辑。
- 不移除 `layout_engine` 导出，也不放宽 Shader06 GPU 测试。

## 修复结果与回传

- 根因：Text07 intermediate re-export named a non-existent provider function
- 架构修复：Removed the invalid re-export and retained the single existing rich-inline layout implementation without shims
- 验证：Current lib test compile passed; Shader06 IBL GPU focused suite passed 39/39
- 回传：Shader06 EC-M3 GPU dispatch/readback gate resumed and passed
