---
handoff_kind: failure
status: open
created_at: 2026-07-18
summary_slug: layout-prefix-and-grapheme-remeasurement
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/text/03-line-breaking-measure-and-layout.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/text/03
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/text/hard_line.rs
  - zircon_runtime/src/text/layout
  - zircon_runtime/src/text/layout/advance_index.rs
  - zircon_runtime/src/text/layout/line_break/boundary_correction.rs
  - zircon_runtime/src/text/layout/line_break/boundary_correction/tests.rs
  - zircon_runtime/src/text/layout/rich.rs
  - zircon_runtime/src/text/layout/rich/materialize.rs
  - zircon_runtime/src/text/layout/rich/metrics.rs
  - zircon_runtime/src/text/layout/rich_advance_index.rs
  - zircon_runtime/src/text/layout/rich_advance_index/tests.rs
  - zircon_runtime/src/text/layout/rich_vertical.rs
  - zircon_runtime/src/text/layout/rich/tests.rs
  - zircon_runtime/src/text/layout/rich_vertical/tests.rs
  - zircon_runtime/src/ui/text/layout_engine/wrapping.rs
  - zircon_runtime/src/ui/text/layout_engine/wrapping/tests.rs
  - zircon_runtime/src/ui/text/layout_engine/rich_inline.rs
  - zircon_runtime/src/ui/text/layout_engine/rich_inline_vertical.rs
  - zircon_runtime/src/ui/text/layout_engine/tests/soft_hyphen.rs
  - zircon_runtime/src/ui/text/layout_engine
  - zircon_runtime/src/text/rich
---

# Text layout增长前缀重测与rich逐字素重塑形

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：`zircon_runtime/src/text/layout/**`当前源20/20 Rust文件
- 修复责任计划：`docs/plans/zircon_runtime/text/03-line-breaking-measure-and-layout.md`
- 责任切片：LB-M1/LB-M3/LB-M4；rich run ownership联动Text07。
- 交接原因：真实range measure、断行、overflow和horizontal/vertical line layout均归Text03；需要统一cluster/BIDI/soft-hyphen语义，不能在性能计划另建第二套布局算法。

## 失败现象与复现证据

PERF-MVP-236：plain path已shape整行，grapheme width仍为每个range重扫全部glyph；greedy wrapping和start/middle/end ellipsis反复构造增长候选String并完整shape/measure，长文本最坏O(G²)。

PERF-MVP-237：rich horizontal/vertical wrap为每个grapheme线性查run、clone style并单独shape；source-range与word chunk重复执行。每个最终视觉行还扫描所有runs并clone整份source text到临时parse result，复杂度O(G×R + L×(R+T))。静态证据见`docs/plans/performance/01/2026-07-18-text-layout-static-review.md`。

PERF-MVP-300/303补充UI adapter证据：newline segment的per-String allocation已局部修复，但block paragraph仍按每line重复prefix `rfind`、全paragraph override扫描、run DTO切片复制；ShrinkToFit/Clamp miss固定执行natural measure加8轮完整measure，ellipsis/rich-inline逐line重复候选shape。证据见`docs/plans/performance/01/2026-07-18-runtime-ui-text-static-review.md`。

## 最低共享层根因

layout owner没有一份可供measure、wrap、overflow和line materialization共同消费的indexed shaped paragraph。上层只拿到可重复调用的shape provider，于是各helper把“测一个range”实现为重新遍历或重新shape字符串；rich style/run identity也没有编译为单调span索引。

## 架构修复验收

- 一次paragraph/style-run shaping产出排序cluster boundaries、source/visual ranges与prefix advances；plain和rich共享range query，不复制glyph DTO。
- `measured_grapheme_widths`单遍投影全部grapheme，禁止每grapheme扫描全部lines/glyphs。
- greedy wrapping累计chunk range/advance，不分配`current+next`增长字符串；soft-hyphen suffix和kerning/ligature边界使用明确的有界修正。
- ellipsis先shape source与ellipsis，再在cluster prefix/suffix index上找边界；start/middle/end不得逐候选完整shape，middle不得每轮重复测左右完整String。
- rich runs编译为排序span与style identity；wrap以cursor/two-pointer消费，same-style连续span只shape一次。inline object保持独立advance/baseline slot。
- line materialization借用原始source并保存source/run ranges；每行不得clone完整`parsed.text`或filter全部runs。
- paragraph/override/run spans用单调cursor消费；每line不得从正文头或全部paragraph/run重新查约束。fit/ellipsis复用同一cluster advances，记录并限制backend calls/iterations。
- 1/100/1k/10k grapheme和1/100/1k runs记录backend calls、glyph/run visits、style clones、owned candidate/source bytes、p50/p95；plain近O(G)或O(G log G)，rich近O(G+R)，每连续style span shape不超过一次。
- UAX14/kinsoku/word-smart/soft-hyphen/tab/justify、ligature/kerning/emoji/combining、RTL/BIDI、VerticalRl、inline metrics、source/run indices、首中尾ellipsis与产品像素等价。

## 禁止临时方案

- 不得用每Unicode scalar固定advance替代真实cluster shaping，或以此让复杂度测试变绿。
- 不得只给候选String/Vec预留capacity；增长前缀重shape与全glyph/run扫描必须消除。
- 不得用无界memoization缓存所有prefix String；索引必须绑定canonical shaped paragraph及font/style generation。
- 不得为rich另建与plain不同的shape backend或丢失跨grapheme kerning/ligature。

## 修复结果与回传

Open state: 本失败仍未关闭；非验收算法实现与二次静态审查已完成，当前只等待 managed coordinator 执行类型/行为/规模/WGPU 验收。不把待验证状态写成 blocked，也不提前记为已验收。

- 2026-07-29 已完成 plain `measured_grapheme_widths` 的单次 shaping 投影：先建立 source grapheme 边界，再由每个 glyph 以二分定位其重叠区间，消除了“每个 grapheme 重扫全部 glyph”的路径；视觉顺序 glyph 到 source 顺序 advance 的回归用例已加入。
- 2026-07-29 已将 end/start/middle ellipsis 的保留边界和 word-end trim 收敛到 `text/layout/overflow.rs` 的 shared advance owner；UI adapter 只投影已确定的 source/run 范围，不再构造并重测逐候选字符串。
- 2026-07-31 已完成 rich horizontal/vertical shared advance index：按排序 source grapheme 保存 advance/cross extent 和 prefix advance；连续相同 `TextStyle` span 只 shape 一次，inline object 保持独立 slot，横排/竖排换行与竖排列输出不再逐 grapheme 重塑形。
- 2026-07-31 已完成 rich line materialization cursor：完整 `parsed.text` 只借用不 clone，视觉行以单调 `run_cursor` 消费相交 runs，保留 original run index，并复用相同 `RichAdvanceIndex` 投影 item advance。该时点的 exact single-span provider-count 回归已在 2026-08-01 被正确的 bounded boundary-window/linear-call 断言取代。
- 2026-07-31 结构优先复审已把 owner 拆为 `rich.rs` 303 行、`rich/materialize.rs` 258 行、`advance_index.rs` 166 行、`rich_advance_index.rs` 246 行、`rich/metrics.rs` 77 行、`rich_vertical.rs` 240 行；均低于当前 production 800 行 review warning，且 production `panic!`/`unwrap()`/`expect()` 与旧 per-grapheme helper 扫描均为 `0`。`rustfmt --check`、tracked/no-index whitespace 静态检查通过。
- 2026-07-31 已完成 greedy wrap 累计入口：plain/rich 共享 `GraphemeAdvanceIndex` 的排序 metric/prefix owner；UI Word 按 chunk range 求和、Glyph 遍历预计算 metric，并携带 `current_advance`。旧 `current_text + next_text` 增长 String、完整前缀 reshape 和 provider-based glyph fallback admission 已删除。该时点的 exact-one miss 断言已在 2026-08-01 按正确的行边界修正语义升级为有界线性预算。
- 尚待完成：managed Cargo 类型/行为/规模回归执行与 p50/p95，以及产品 WGPU framebuffer 验收；这些不能用当前静态证据替代。
- 验收顺序：由 coordinator wakeup 后运行 Text03 focused/upward tests 和规模计数，再执行 Text09 缓存与真实 WGPU 产品 framebuffer 截图。等待 managed 槽只延迟 accepted closeout，不阻止继续完成其他可落地非验收任务。
- 2026-08-01 已完成 bounded boundary owner：plain/rich/UI horizontal/VerticalRl Glyph 与 Word/WordSmart 都复用 prefix advance，只物化首尾各 16 个 context units 的 bounded indexed query；单次 shape window 为 16 graphemes（soft-hyphen suffix 另计），不再收集增长中的完整候选行。plain/rich 重复 planner 已合并。
- 2026-08-01 soft-hyphen 已形成测量到绘制闭环：合法 chunk 断点以 suffix 修正宽度，普通 UI 使用 pending suffix，rich-inline horizontal/VerticalRl 追加 synthetic `-` run、隐藏 U+00AD source range 和真实 advance。1/100/1k/10k grapheme backend-call/window、1/100/1k alternating rich-run single-shape 和 UI token cache-miss 线性回归已写入源码。
- ignored `boundary_scale_evidence_reports_p50_p95` 已就绪，按 31 samples 输出各 grapheme 规模的 line count/backend calls/max window/p50/p95 ns，不用机器时延阈值伪造正确性 pass。
- 2026-08-01 二次静态审查已修复重复 planner、O(G^2) edge-unit materialization 和 rich suffix 漏投影三项缺陷；owner 均低于 800 行，格式/whitespace/旧增长候选/production panic-unwrap-expect-dead-code allow 扫描通过，未留下 actionable P0/P1/P2。
- 2026-08-01 Text02实现完成后的定向二次审查发现Text03 hard-line消费仍有偏差；现已由公共`text/hard_line.rs`统一CR/CRLF/LF/VT/FF/NEL/LS/PS，measure/rich/UI与shaping共用同一content/separator range。line-break chunk改用absolute source range并保留mandatory标记，kinsoku/word-smart禁止跨强制边界合并；BIDI line order只计算一次reordered levels。对应回归已写入，managed Cargo仍待coordinator，failure不提前标记fixed。
- 当前 failure 保持 `open` 的原因已从“算法实现未完成”收窄为 managed focused/upward Cargo/规模回归执行与 p50/p95、Text09 cache 回归和新鲜 WGPU 产品 framebuffer 尚未验收。状态为 `implementation_complete / resolving_failure / managed_validation_pending`，不是 blocked，也不允许用旧 PNG 或策略图关闭。
- Coordinator handoff：validation submit 因 Session numbered-Plan registration 尚未物化而被拒绝/health-preflight timeout，未产生 queued/running ticket；完整 registration 已收到 durable accepted receipt `a6d7f08e10c24387be4c8b73611319e6`。按规则不轮询 receipt，coordinator wakeup 后前向提交 `boundary`、`rich_span_index`、`soft_hyphen`、ignored p50/p95 exporter 与 ignored exact WGPU product 命令。
- 2026-08-03 SDF source-map 前向修复已完成 visual-order fast path 回归：layout 物化 `"A בא"` 时与 advances 同序重排，数量未改变的 glyph run 直接使用 resolved layout advances；只有 ligature、mark 或 virtual-text 改变 source-range topology 才进入 range projection。`rustfmt --edition 2024 --check`、scoped whitespace 及 production 禁用模式扫描通过。failure 继续保持 `open / implementation_complete / resolving_failure / managed_validation_pending`；权威 WGPU PNG 尚不存在。
- 2026-08-03 定向二次审查确认 horizontal shaped-SDF 的 layout advance 投影覆盖 tatweel、visual Bidi、ligature、combining-mark 和 rebased virtual source range；VerticalRl shaped glyph 已在 render extract 阶段消费同一 resolved main-axis advance，不能机械套用 horizontal projection。新增回归保留在独立 `sdf_render/tests/shaped_advances.rs` owner，生产 leaf 为 164 行；未发现本切片 actionable P0/P1/P2，受管 Cargo/真实 WGPU PNG 仍是唯一未验收项。
