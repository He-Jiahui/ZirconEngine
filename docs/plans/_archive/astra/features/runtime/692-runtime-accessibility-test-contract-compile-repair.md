---
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/78-runtime-ui-accessibility-semantic-tree-name-description-relation-state-action-live-region-platform-adapter-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/82-runtime-text-editing-document-selection-caret-hit-test-ime-composition-clipboard-secure-text-product-integration-current-source-review.md
related_code:
  - zircon_runtime/src/ui/tests/accessibility_state_values.rs
  - zircon_runtime/src/ui/tests/accessibility_text_input_actions.rs
  - zircon_runtime/src/ui/tests/accessibility/value_actions.rs
---

# Runtime Accessibility 测试合同编译修复

历史受管 Windows UI-feature 编译日志定位到 accessibility 测试的两类漂移：

- secure text input 断言从临时 `accessibility_snapshot()` 借用 node，快照在断言前已析构；
- composition-clear 断言期待旧 slice 类型 `Option<&[_]>`，但当前 TOML API 返回
  `Option<&Vec<toml::Value>>`。

本批次将两处 snapshot 保存为局部绑定，并为三处空 composition-clause 断言保留具名、类型化
的空 `Vec<toml::Value>`。它不改变 secure text redaction、accessibility role/action、
text selection、IME composition 或任何 Runtime 生产路径。

## 计划完成列表

| 批次 | 内容 | 状态 | 验证证据 |
|---|---|---|---|
| Runtime78 / Runtime82 | secure-text snapshot lifetime 与 composition-clause assertion 合同编译修复 | `implemented_pending_validation` | 历史受管日志定位两项 temporary-borrow 与三项 expected-type 诊断；当前源码守卫 `5/5` 且确认无旧空 slice 断言、scoped diff-check 通过。并行静态批次：Runtime performance-contract `1146/1146`（`4.114s`）、Editor performance-contract `581/581`（`1.051s`）、Runtime Text contracts `143/143`（`1.159s`）。受管 Windows Cargo、原始 Rust 回归和 Release p50/p95/p99 仍待异步批量验证。 |

整文件 Rustfmt 仍报告这些测试中本次未触及的既有格式差异，因此没有将合同修复扩大为格式化重写；静态批次不替代产品性能验收。
