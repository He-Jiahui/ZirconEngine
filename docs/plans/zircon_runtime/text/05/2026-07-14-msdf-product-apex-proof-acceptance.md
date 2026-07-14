# Text05 MSDF 产品尖角证明验收

Plan: docs/plans/zircon_runtime/text/05-sdf-msdf-pipeline.md
Milestone: M2
Status: accepted
Files: ["zircon_runtime/tests/runtime_text_multilingual_product_framebuffer.rs", "zircon_runtime/tests/runtime_text_multilingual_product_framebuffer/proof_assertions.rs", "docs/zircon_runtime/graphics/text/sdf.md", "docs/plans/zircon_runtime/text/05/2026-07-09-sdf-msdf-pipeline-output-records.md", "docs/tests/runtime/text/runtime_text_multilingual_sdf_msdf_product_framebuffer_20260714.png"]

## Scope delivered

| 里程碑 | 状态 | 结论 |
|---|---|---|
| M2 | 通过 | 产品级证明不再使用 24px 栅格中不稳定的“MSDF 前四行高对比像素数必须严格多于 SDF”作为几何优越性代理；两路均须产生真实尖角像素、decode 结果须不同，且 MSDF 顶点不得低于 SDF。renderer-neutral fdsm 回归继续单独拥有 bake-space 几何准确性。 |

## Fresh testing evidence

- RED：旧产品断言在真实 WGPU 帧中得到 `sdf=19, msdf=19`，1 failed / 0 passed，1009.95s；历史 2026-07-13 验收帧也只以 `21/22` 的单像素差通过，证明该指标受网格量化影响。
- GREEN compile：managed job `017de9fb45784ec396ec90dcd20f6584`，`cargo test -p zircon_runtime --test runtime_text_multilingual_product_framebuffer --no-run --locked`，exit 0，9m06s。
- GREEN product：exact ignored `export_runtime_multilingual_text_product_framebuffer_png`，1/1 passed，1017.34s。
- 当前 manager 边界：integration test 通过 `ProjectAssetTestRuntime` 注册并解析 `ProjectAssetManagerAccess` 后构造 `WgpuRenderFramework`；没有恢复旧 `Arc<ProjectAssetManager>` 构造路径。
- 结构闸门：integration root 800 行；scoped `rustfmt --edition 2021 --check` 通过。

## Review

原尺寸人工检查通过，关键产品证据与产物卫生如下。

- 路径：`docs/tests/runtime/text/runtime_text_multilingual_sdf_msdf_product_framebuffer_20260714.png`
- 尺寸：1080×1690
- 大小：321453 bytes
- 颜色：2442
- SHA256：`2A033D76EF5C16F99FB6B256AD8F480ACE494FB03537A9E4502DEA293BED866E`
- 原尺寸检查：多语言、RTL、emoji、VerticalRl、BBCode/table、inline texture、SDF/MSDF `A/M/W` 均为真实 framebuffer 像素；不是策略文字截图。
- 产物卫生：repository `target` 以及 D/E/F approved Cargo target roots 中同名文件为 0。
