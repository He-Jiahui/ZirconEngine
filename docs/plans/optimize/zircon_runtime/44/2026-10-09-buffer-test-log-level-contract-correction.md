# Runtime44 · 缓冲回归的日志级别字节合同修复

日期：2026-10-09。承接[缓冲复用记录](2026-10-09-bounded-worker-batch-buffer-reuse.md)。本次修复新回归用例的错误字节期望，源码已实际接入，native 和性能保持待验证。

## 原因与改动

现有 `DiagnosticLogLevel::as_str` / `Display` 输出 `log`、`warn`、`error`。真正 `append_diagnostic_log_line` 以 `[{level}]` 写入头部，原单行及多记录 golden 均采用小写。新 `batch_buffer_tests.rs` 有四处大写期望：LOG 一处、WARN 两处、ERROR 一处，导致该新回归无法符合既有格式合同。问题在源码审查中发现，尚无本次 native 执行结果。

修复只将四处期望对齐原小写合同。两个 active cases、其余断言、成功/失败输出、完整超大记录、容量释放、FIFO 与 metrics 检查保留。生产 formatter 和原 golden 文件没有修改。

## 实际证据

- 单文件 patch SHA-256：`156b0669d5cabfe2349d57a87013417346dc5b1ffbdfda2b02e5310350fc11df`。
- 独审 `runtime44-buffer-test-level-case-correction-core-review-v1/review.json` SHA-256：`71f663d7a172ad6f02825e9f574fcb44dcab47c713f10180416c1540047c47f0`，源码接受，native pending。独审更正旧 `dc0abbe45956ed20ca4fb9e4b490378b6c930fbc2e607e9821ca461dc396d025` 漏核新期望与真实 Display 的问题；旧报告保留。
- 实际接入 `.codex/outbox/runtime44-buffer-test-level-case-correction-v1/actual-canonical-apply.json` SHA-256：`e04b4d96f582fc28ae214af1f6fc5d8878445017616f582e063ef58872c23b44`。六项 fresh raw 输入匹配，五项未修改依赖保持；单次 apply check/apply、实际 Rust 2021 fmt、scoped diff exit 0；共享 index、config、Cargo 输入未变。
- 实际 LF 后像：`7097c721cccdcfef5f1f4050c23c6eb5456d9b23e55cd62af41900ba3e3f0143`。

## 同批待验收

原 Runtime #2 已包含两个缓冲新例、原 golden/batching/durability/FIFO 以及五项 shutdown/join/TLS 门。本次用例数和完整名字不变，最终批次必须绑定纠错后的源码。原十五项配置、Runtime44 Release 54 行矩阵、critical admission 20 个样本及其原阈值保留。

仍需实际 native 成功、原完整性能矩阵与真正产品验收。原 buffer 接入回执及其历史后像保留；当前测试文件以本次后像为准。详见[Runtime 完成列表](../../../astra/features/runtime/2026-10-09-runtime44-buffer-test-log-level-contract-completion-list.md)。
