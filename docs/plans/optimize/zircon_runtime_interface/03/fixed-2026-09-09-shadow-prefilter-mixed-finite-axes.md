---
handoff_kind: fixed
failure_scope: local
status: fixed
created_at: 2026-09-08
summary_slug: shadow-prefilter-mixed-finite-axes
origin_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
fixing_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
origin_child_dir: docs/plans/optimize/zircon_runtime_interface/03
fixing_child_dir: docs/plans/optimize/zircon_runtime_interface/03
plan_link_mode: child_record_only
related_code:
  - zircon_runtime_interface/src/ui/surface/render/text_effects.rs
  - zircon_runtime_interface/src/ui/surface/render/text_effects/performance_tests.rs
tests:
  - cargo test -p zircon_runtime_interface --no-default-features --locked --lib ui::surface::render::text_effects::
  - cargo test -p zircon_runtime_interface --no-default-features --locked --release --lib runtime_interface03_batch55_61_prefiltered_text_effects_release_benchmark -- --ignored
  - cargo test -p zircon_runtime_interface --no-default-features --locked --lib
resolved_at: 2026-09-09
---

# Interface03: shadow prefilter must preserve independent axis normalization

## 来源执行者

- 来源计划：`docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md`
- 来源执行切片：完整接口库中 text-effect normalization 的动态回归。
- 修复责任计划：`docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md`
- 交接原因：公共 text effect 归一化由 Interface03 拥有，渲染者消费其 normalized payload。

## 失败现象与复现证据

受管 Windows 作业 `f95f64a6a06345d3940884140d9e3e50` 在完整接口库执行
`text_effect_contract_normalizes_non_finite_and_out_of_range_values`，得到 shadow `None`。
输入 `offset_x_px=-128`、`offset_y_px=NaN`、颜色 `#11223344` 应独立归一化为
`(-64,0)` 的可见阴影。作业共 739 passed、23 failed、101 ignored；
manifest `484b58e5512bb5619941864b4906bbfaaeef0cff78f89267586fe6e4b00d2b63`，
日志位于受管 `interface-library-consumers-3137-20260908/results/interface-library-3137.log`。
后继 `b6fbfec9ae5f4dc0bf766460167377a4` 也保留此失败，完整结果 744 passed、18 failed、101 ignored。

## 最低共享层根因

Batch55-61 的 inactive-effect 预过滤在任一原始坐标非有限时丢弃整个 shadow。
原有 normalize-then-filter 契约先分别把非有限坐标归零、有限坐标限到正负 64，再判断
是否还有有效位移，因此 mixed-finite axes 不能直接整体丢弃。

## 架构修复验收

- 原始 `(-128,NaN)` 回归通过，保留 `(-64,0)` 和 trimmed RGBA 断言。
- 192 组坐标与颜色组合与独立的旧 normalize-then-filter oracle 等价，覆盖 NaN、正负
  Infinity、零、正负 epsilon、正负越界值、透明色、空色与非空色。
- 既有 release 基准执行 250,000 次、11 组交替采样，保留 P95 至少改善 20% 的门槛。
- 完整接口库、独立审查与正式 fixing-Session closeout 绑定完成。

## 禁止临时方案

- 不放宽或删除旧结果断言，不跳过 release 门槛，不恢复 inactive color 的无谓 String 分配。
- 不将静态 guard 通过或全库中的通过子集当作正式 failure 验收。

## 修复结果与回传

- 根因：The inactive text-effect prefilter discarded the whole shadow whenever either raw offset axis was non-finite, violating the existing normalize-then-filter contract for mixed finite axes.
- 架构修复：The prefilter now normalizes each signed extent independently before deciding visibility, preserving color prefiltering and delayed allocation; the shared normalized payload remains the sole consumer contract and no fallback String allocation was restored.
- 验证：Managed Windows interface job 26fad5951ec5484ab28b327f5e53c597 executed the mixed-finite regression and 192-coordinate/color oracle with 746 passed, 17 failed and 101 ignored overall. Managed release job de1e0dc24fd84ebd8350a0a39a9a1128 executed the 250000-iteration, 11-group benchmark gate with 1 passed, 0 failed, 0 ignored and 863 filtered. The worker did not emit numeric P95, so no numeric improvement is claimed. Independent review recorded Critical 0, Important 0, Moderate 0.
- 回传：Returned text-effect normalization at text_effects.rs 33f7423f8d5c0ec405af527446d8c3b0190895aa24e47e0b35df60fe01cb57ee and performance_tests.rs a4271883453f385e0e774c3d393185097aac1eaeebe840f17d0038fbbbce5ed6. Mixed finite axes and the existing release threshold test are green; remaining full-library failures and missing numeric P95 output stay explicit.
