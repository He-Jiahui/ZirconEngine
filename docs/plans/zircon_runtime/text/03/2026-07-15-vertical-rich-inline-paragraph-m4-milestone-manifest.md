Plan: docs/plans/zircon_runtime/text/03-line-breaking-measure-and-layout.md
Milestone: M4
Status: completed
Files: ["docs/plans/zircon_runtime/text/03/2026-07-09-line-breaking-measure-and-layout-output-records.md","docs/plans/zircon_runtime/text/03/2026-07-15-vertical-rich-inline-paragraph-m4-milestone-manifest.md","docs/plans/zircon_runtime/text/03/fixed-2026-07-15-runtime-operation-ffi-sibling-visibility.md","docs/tests/runtime/text/runtime_text_vertical_rich_inline_paragraph_product_framebuffer_20260715.png"]

# Text03 M4 VerticalRl paragraph rich-inline composition 产品验收清单

## Scope Delivered

M4 已提交实现把 physical paragraph 的 first/continuation usable height、indent 与 alignment 组合进共享 VerticalRl rich-inline wrapping。中立 graphics owner 使用 inline object height 作为主轴 advance；UI owner 统一预解析 paragraph constraints 并供 plain/rich-inline 路径复用，未引入 renderer reconstruction、post-layout move、兼容 facade 或重复 truth。本轮 testing-stage closeout 验收 node 120 在真实 VerticalRl 段落内消费 imported checker texture，并只把产品 framebuffer 写入 `docs/tests/runtime/text`。

## Fresh Testing Evidence

- 实现切片 exact rustfmt、scoped diff 与独立代码复审均已通过（P0=0、P1=0、P2=0）。
- Windows 受管默认功能聚焦过滤实际运行五条 `bbcode_inline_image_vertical_rl*` 回归：5 passed / 0 failed。
- exact ignored product exporter 实际运行：1 passed / 0 failed；测试体 483.00s，经过 production SDF atlas、WGPU submit 与 framebuffer readback。
- exporter 断言覆盖 per-row framebuffer delta、VerticalRl resolved layout、U+FFFC inline run、object-height advance 与 imported checker texture channel pixels。
- 产品 PNG：`docs/tests/runtime/text/runtime_text_vertical_rich_inline_paragraph_product_framebuffer_20260715.png`，1080×1840，354117 bytes，SHA256 `A5B38D81F8ACA85BE826AC87E441E73A120437CAE12E7D769559FDABC681E77E`。
- 原图目检确认右侧 VerticalRl 多列文字、首列缩进、段落对齐与 checker inline object 均可见；该图是产品 framebuffer，不是纯文本策略目标图。
- repo 同名文件只命中上述 docs 路径；`D:\cargo-targets` 与 `E:\cargo-targets` 同名扫描均为 0。

## Review

实现 reviewer 已完成两轮检查。首轮发现空 physical paragraph source range 匹配过宽和 Word/WordSmart 证据强度不足；修复后复审为 0 Critical / 0 Important / 0 Minor。本轮 acceptance reviewer 还需核对截图来源、数字、hash、target 排除与清单 scope 后提交协调器 workflow review。

## Acceptance Boundary

Editor03 operation bridge 的 sibling visibility 已保持为 crate-internal `pub(crate)`，验证 handoff 已回传为 `fixed-2026-07-15-runtime-operation-ffi-sibling-visibility.md`。本清单关闭 M4 当前 VerticalRl paragraph rich-inline 产品验收切片；计划仍保持 `in_progress`，继续推进全量 native/SDF paragraph parity、复杂 vertical source-range geometry 与平台实机文字输入。
