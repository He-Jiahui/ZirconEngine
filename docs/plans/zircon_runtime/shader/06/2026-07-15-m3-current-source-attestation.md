# Shader06 M3 Current-Source Attestation

Plan: docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md
Milestone: M3
Status: completed
Files: ["docs/plans/zircon_runtime/shader/06/2026-07-15-m3-current-source-attestation.md"]

## 状态与产出记录

| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| M3 | 当前源码 IBL artifact 与 source staging 重新证明 | `completed` | 2026-07-15 | Windows managed job `03c5072d8e5b4214b146fd620027324d` 在当前共享源码上通过 `runtime_environment_ibl_bake_artifact_contract` 23/23 和 `runtime_environment_ibl_source_import_staging_contract` 6/6（1 ignored）；同一 job 继续通过 viewer 18/18，证明验证后工作区可向上编译。 |
| M3 | 历史清单完整性合规处理 | `completed` | 2026-07-15 | 旧 accepted M3 commit `f6f9cf8f29c60976288353268c9319c399276ffd` 保持不可变；缺失逐文件证明的旧 manifest hash 不被覆盖、放宽或伪造。本记录以新的一文件 manifest 提供当前源码验证性证明。 |

## Scope Delivered

- 仅重新证明当前源码的 M3 IBL bake artifact、独立 PMREM layout、derived/cache/runtime dispatch/writeback 和 source import staging 合同。
- 不修改 M3 生产实现、不重提旧源码、不改变历史 commit 或 recorded manifest hash。
- 后续 M4/M5 使用各自独立 current-source attestation 和产品证据，不把本记录扩大为其完成声明。

## Fresh Testing Evidence

- Managed job: `03c5072d8e5b4214b146fd620027324d`, target `D:\cargo-targets\zircon-engine\pool\c07cadc864b35086ee68c4f87411d5a2a854b0e5f37ed02c5b10c87e4873aca6`, exit 0.
- `cargo test -p zircon_runtime --test runtime_environment_ibl_bake_artifact_contract --locked -- --test-threads=1`: 23 passed, 0 failed, 0 ignored.
- `cargo test -p zircon_runtime --test runtime_environment_ibl_source_import_staging_contract --locked -- --test-threads=1`: 6 passed, 0 failed, 1 ignored.
- `cargo test -p zircon_app --bin zircon_shader_pbr_viewer --locked -- --test-threads=1`: 18 passed, 0 failed, 0 ignored.
- Earlier job `65ec4b69019f4f55a594d58e3796d77d` passed the first 23 tests but was correctly rejected as a complete gate when concurrent Text DTO drift blocked the second target; it is not the acceptance job.

## Review

This verification-only record preserves the accepted M3 implementation commit while replacing an unrecoverable historical proof gap with current-source evidence. An independent reviewer reran the three current-source test targets and reported Critical `0`, Important `0`, and Minor `0`. The coordinator must still record that result through the milestone review gate before commit.
