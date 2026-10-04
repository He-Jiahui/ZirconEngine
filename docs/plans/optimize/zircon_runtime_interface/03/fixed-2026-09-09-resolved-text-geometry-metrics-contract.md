---
handoff_kind: fixed
failure_scope: local
status: fixed
created_at: 2026-09-08
summary_slug: resolved-text-geometry-metrics-contract
origin_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
fixing_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
origin_child_dir: docs/plans/optimize/zircon_runtime_interface/03
fixing_child_dir: docs/plans/optimize/zircon_runtime_interface/03
plan_link_mode: child_record_only
related_code:
  - zircon_runtime_interface/src/ui/surface/render/text_geometry/source_map/cluster.rs
  - zircon_runtime_interface/src/ui/surface/render/text_geometry/source_map_tests.rs
  - zircon_runtime_interface/src/tests/render_contracts.rs
tests:
  - cargo test -p zircon_runtime_interface --no-default-features --locked --lib ui::surface::render::text_geometry::
  - cargo test -p zircon_runtime_interface --no-default-features --locked --lib tests::render_contracts::
  - cargo test -p zircon_runtime_interface --no-default-features --locked --lib
resolved_at: 2026-09-09
---

# Interface03: resolved text geometry must respect metric and glyph authority

## 来源执行者

- 来源计划：`docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md`
- 来源执行切片：完整接口库中的 editable paint 与 source-map 回归。
- 修复责任计划：`docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md`
- 交接原因：visual/source cluster 几何与公共 paint 投影由 Interface03 拥有；真实 glyph/face
  shaping authority 继续归属 Runtime Text，不由 resolved layout 推测。

## 失败现象与复现证据

受管 Windows 作业 `f95f64a6a06345d3940884140d9e3e50` 实际执行完整接口库，
739 passed、23 failed、101 ignored。此链包含四项失败：

- `invalid_glyph_advances_expose_only_known_line_endpoints` 在 cluster builder 的 debug 断言 panic。
- `ui_text_paint_carries_editable_caret_selection_and_composition` 没有预期 selection 几何。
- `ui_text_decorations_snap_to_grapheme_cluster_edges` 没有预期 grapheme selection 几何。
- `ui_text_decorations_use_run_visual_ranges_for_non_isomorphic_source_ranges` 要求 layout
  生成 canonical shaped artifact，与当前规范 authority 冲突。

输入 `interface-library-consumers-3137-20260908` 的 manifest 为
`484b58e5512bb5619941864b4906bbfaaeef0cff78f89267586fe6e4b00d2b63`；
日志和受管 receipt 保留在该输入的 `results/interface-library-3137.{json,log}`。

## 最低共享层根因

cluster builder 对公开 DTO 的非空 advance 数组强制长度相等，在 debug 下抢先 panic。
`UiTextLineSourceMap::new` 本身已经统一校验完整长度、有限值和非负性，并且在无效数据上
仅暴露已知行端点；提前断言使该防御路径和其现有回归无法执行。

两个 editable fixture 仅给出总宽度，却要求精确内部 caret/selection 几何。
它们应提供每个 grapheme 的实际 advances。第三个 fixture 已有精确 advances，
其原几何断言通过，但它仍错误要求 resolved layout 伪造 Runtime 的 canonical glyph artifact。

## 架构修复验收

- malformed advance 回归保持全部断言并通过，长度不足、NaN、负数均不 panic，内部几何不插值。
- 两个 fixture 提供精确 `10 * 5` 和 `15 * 2` advances，原 selection/composition/caret 坐标完全保留。
- non-isomorphic soft-hyphen 源区间仍为 3..5，paint run frame 仍为 `(30,0,10,12)`，
  canonical shape 明确为 `Unavailable`；真实 glyph 生产者未改变。
- source-map、editable render 及完整接口库门禁复验，独立审查完成，正式 closeout 绑定有效。

## 禁止临时方案

- 不用均匀估算替代缺失 glyph advances，不伪造 glyph/face，不放宽原几何断言或 ignore 测试。
- 不把已知行端点的 fail-closed 行为替换为不受限 fallback，不改变性能测试门槛。

## 修复结果与回传

- 根因：The text source-map cluster builder asserted that non-empty advance arrays matched the declared glyph count before the existing metric validation could fail closed, and editable fixtures lacked per-grapheme advances while one non-isomorphic fixture demanded resolved layout to invent Runtime glyph authority.
- 架构修复：Removed the premature debug assertion, retained centralized finite and non-negative metric validation with known line endpoints, supplied exact advances for editable fixtures, and changed the non-isomorphic case to assert canonical shape Unavailable while preserving paint-run geometry; Runtime text remains the glyph producer.
- 验证：Managed Windows interface job b6fbfec9ae5f4dc0bf766460167377a4 executed all four original text metric/geometry regressions and they passed; full library result was 744 passed, 18 failed and 101 ignored. Existing independent review reported Critical 0, Important 0, Moderate 0; unrelated full-library failures remain explicit.
- 回传：Returned the resolved text geometry repair at cluster a563604feb7844510ca613b4424c510e157d603f56d404f4e9146b3ccdef7a61, source-map tests a9caef847cc1dbb0c6e17a2bdd7c7d3c246639b62115655e612506e1a25ee9f4 and render contracts 6a3a51ad1c4808e6420030de70377c98a32f365e0ab87766f989873178022d76. Malformed metrics fail closed without interpolation or fabricated glyph data.
