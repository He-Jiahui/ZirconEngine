Plan: docs/plans/zircon_runtime/text/03-line-breaking-measure-and-layout.md
Milestone: M4
Status: completed
Files: ["docs/plans/zircon_runtime/text/03/2026-07-09-line-breaking-measure-and-layout-output-records.md","docs/zircon_runtime/graphics/text.md","docs/zircon_runtime/ui/text/layout_engine.md","zircon_runtime/src/graphics/text/layout/rich_vertical.rs","zircon_runtime/src/ui/text/layout_engine/paragraph_layout.rs","zircon_runtime/src/ui/text/layout_engine/rich_inline_vertical.rs","zircon_runtime/src/ui/text/layout_engine/tests/rich_inline.rs","zircon_runtime/src/ui/text/layout_engine/vertical.rs","zircon_runtime/tests/runtime_text_multilingual_product_framebuffer.rs","zircon_runtime/tests/runtime_text_multilingual_product_framebuffer/proof_assertions.rs","zircon_runtime/tests/runtime_text_multilingual_product_framebuffer/proof_commands.rs","docs/plans/zircon_runtime/text/03/2026-07-15-vertical-rich-inline-paragraph-m4-milestone-manifest.md"]

# Text03 M4 VerticalRl paragraph rich-inline composition 里程碑清单

## Scope Delivered

M4 当前实现切片把 physical paragraph 的 first/continuation usable height、indent 与 alignment 组合进共享 VerticalRl rich-inline wrapping。中立 graphics owner 使用 inline object height 作为主轴 advance；UI owner 统一预解析 paragraph constraints 并供 plain/rich-inline 路径复用，未引入 renderer reconstruction、post-layout move、兼容 facade 或重复 truth。产品 exporter 已改为在 node 120 的真实 VerticalRl 段落内消费 imported checker texture，截图目标只允许 docs/tests/runtime/text。

## Fresh Testing Evidence

- exact rustfmt check：通过。
- scoped git diff check：通过。
- 独立代码复审：P0=0、P1=0、P2=0。
- managed Windows focused run：5 tests 中 4 passed；Word/WordSmart 一条暴露测试期望 ab 与实测合法首列 abc 的偏差，断言已校正为 abc 并继续由 18px usable-height 上限锁定。
- 校正后的 fresh rebuild 在 Text03 执行前被 zircon_runtime_interface/src/runtime_api/operation.rs:60 的并发 E0424/E0369 阻断；已创建并导入 Runtime02 open failure runtime-operation-phase-terminal-pattern。当前不把 focused 5/5、WGPU exporter 或 PNG 冒充通过。

## Review

独立 reviewer 已完成两轮检查。首轮发现空 physical paragraph source range 匹配过宽和 Word/WordSmart 证据强度不足；修复后复审为 0 Critical / 0 Important / 0 Minor。协调器 workflow review 将在 manifest 绑定后提交相同结论。

## Acceptance Boundary

本清单用于尽快同步已完成的实现与真实阻断证据，不宣称 M4 产品验收完成。Runtime02 failure 回传后必须复跑 5/5、exact ignored WGPU exporter，并只接受 docs/tests/runtime/text 下经原图目检、尺寸/hash 与 target 排除检查的真实 framebuffer PNG。
