---
title: Runtime75 Batch Push Performance Summary
category: zircon_runtime
report_id: Runtime75-batch-push-performance-summary-2026-08-30
date: 2026-08-30
source_plan: docs/plans/optimize/zircon_runtime/75-runtime-ui-component-catalog-widget-behavior-state-reducer-interaction-semantics-accessibility-product-integration-review.md
implementation_status: implementation_complete
validation_status: local_contracts_green_managed_product_qualification_pending
promotion_status: pushed_not_promoted
---

# Runtime75 批量优化推送记录

## 发布范围

- 分支：`main`
- 推送范围：`f660cfa9f..cdf0663ac`
- 推送提交数：28
- 远端 HEAD：`cdf0663ac`
- 企微通知：已发送，API `errcode=0`

本批次包含 Runtime75 组件目录/调色板/Toast 投影优化及其协调器控制面提交。提交前未将
未提交的 Tooling15 acceptance 改动纳入发布。

## 性能证据

| 场景 | 基线 | 优化后 | 变化 |
| --- | ---: | ---: | ---: |
| Component category projection allocations | 2 | 0 | -100% |
| Component category projection P50 / P95 | 3.150 ms / 5.6137 ms | 0.1073 ms / 0.1287 ms | -96.59% / -97.71% |
| Palette projection allocations | 7 | 1 | -85.71% |
| Toast scan allocations | 3,207 | 3 | -99.91% |
| Toast scan P50 / P95 | 620.2 us / 755.0 us | 86.9 us / 99.1 us | -85.99% / -86.87% |

Random checkpoint evidence (P50/P95, us; sizes 1/64/1024/65536):
`0.5/0.6`, `13.2/15.8`, `211.7/321.8`, `22925/41934`.

## 验证

- Python `test_*performance_contract.py`：`1630/1630` 通过，`16.711s`。
- Tooling PowerShell batch：`5/5` 通过。
- Runtime75 managed contract：`9` tests passed。
- `git diff --check` 和相关脚本 AST parse：通过。

## 测试闭环补充

- 协调器收口提交：`657177e5b6fde13cb799588190bc476f75e98d93`。
- 本提交补回 `material_foundation/shared.rs` 的 `cfg(test)` 所引用的
  `shared/enum_label_tests.rs`，使 Runtime75 的干净源码快照具备完整测试模块；不改变运行时
  性能算法或产品行为。
- 托管 Runtime75 静态验证请求 `d422af283de64c1c9569ceeda1a84a38` / ticket
  `0dddcab62c8d4e81adb9b295226f0efa`：9/9 通过。
- Rust `fmt --check` 和 `git diff --check`：通过。
- Cargo release/debug 验证仍受外部 materialization 治理阻塞：缺失的未跟踪 RHI 源文件、受管
  目标目录中的 unmanaged artifact，以及 `F:\cargo-targets\zircon-engine\ephemeral`；这些
  路径不属于本 Session，未修改或认领。
- 该收口提交继承本批次性能证据，未新增可信 ProductReceipt-bound P50/P95；状态继续保持
  `pushed_not_promoted`，不得宣称最终产品性能资格已完成。
- 收口通知已推送企微（`errcode=0`，2026-08-30 09:39:30 +08:00）。

## 资格边界

以上是已推送提交的局部性能证据，不等同于 Runtime75 全量产品资格。组件 authority、live
surface conformance、10/1k/100k 产品负载，以及 trusted ProductReceipt-bound Cargo/产品
P50/P95 仍待托管验证；在这些证据闭环前，状态保持 `pushed_not_promoted`，不得宣称最终产品
性能达标。
