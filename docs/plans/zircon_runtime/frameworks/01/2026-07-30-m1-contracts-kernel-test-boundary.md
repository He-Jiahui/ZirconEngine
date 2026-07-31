# Frameworks01 M1 contracts/kernel 测试边界硬切

> 承接 [`01-runtime-crate-decomposition.md`](../01-runtime-crate-decomposition.md) M1 Phase 1 的第一个物理迁移前置：`zr_contracts` 不得通过测试反向依赖 concrete `zr_kernel` 实现。

Plan: docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md

Milestone: M1

Status: validation_pending

Files: ["tools/tests/test_frameworks_01_contracts_kernel_test_boundary.py", "zircon_runtime/src/core/framework/render/environment/source_cubemap.rs", "zircon_runtime/src/core/framework/render/environment/source_cubemap/pmrem.rs", "zircon_runtime/src/core/framework/render/environment/source_cubemap/projection.rs", "zircon_runtime/src/core/framework/render/environment/source_cubemap/tests.rs", "zircon_runtime/src/core/framework/render/environment/source_cubemap/tests/projection.rs", "zircon_runtime/src/core/runtime/tests.rs", "zircon_runtime/src/core/runtime/tests/tasks.rs", "docs/plans/zircon_runtime/frameworks/01/2026-07-30-m1-contracts-kernel-test-boundary.md"]

## 范围

- contracts-side owner：`zircon_runtime/src/core/framework/render/environment/source_cubemap/tests/projection.rs`
- kernel integration owner：`zircon_runtime/src/core/runtime/tests/tasks.rs`
- boundary guard：`tools/tests/test_frameworks_01_contracts_kernel_test_boundary.py`
- dependency-complete source owner：`core/framework/render/environment/source_cubemap{.rs,/}` 的 projection/mipmap/PMREM 拆分。
- 不修改 production `ParallelSliceExecutor` 契约，不给未来 `zr_contracts` 增加 `zr_kernel` dev-dependency，不保留旧测试副本或兼容入口；并行 equirect sampler 直接硬切为真实并发所需的 `Fn + Send + Sync`。

## 当前状态

状态：`dependency_complete_exact9 / static_green / managed_rust_gate_pending`

### 已完成项目

- 建立 production+test source 同口径静态守卫，扫描 `core/framework/**/*.rs`，禁止导入 concrete `crate::core::runtime` owner。
- TDD RED 为 `1 failed`，只报告 source-cubemap projection 测试的两处 `TaskPool` import（原第 74、88 行），与 M0 原子依赖基线完全一致。
- 原 r1/exact5 无法形成 clean-HEAD 原子提交：它依赖尚未提交的 source-cubemap tests/module 拆分。该会话已由 `frameworks01-m1-contracts-kernel-test-boundary-r2-20260730` 取代，失败的 validation-copy 尝试均停在 pre-Cargo 控制面阶段。
- r2 已领取并 attribution dependency-complete exact11：M1 guard/kernel tests 加上 archived Render13/Shader06 留下且无活动 owner 的 6 个 source-cubemap production/test dependency 文件；不吸收 importer、viewer、scene 或其他 Session 路径，也不宣称关闭 Render13 的更大 staging handoff。
- 两项 concrete `TaskPool` 集成测试已从 contracts-side source-cubemap test owner 移至 kernel-owned `core/runtime/tests/tasks.rs`，原位置不保留副本或兼容入口。
- `core/runtime/tests.rs` 已挂载新的 kernel integration test module；production `ParallelSliceExecutor` trait 未修改。并行 source-cubemap constructors 的 sampler bound 从串行 `FnMut` 硬切为 `Fn + Send + Sync`，与实际 face-parallel 调用一致，不保留旧 bound/shim。
- boundary guard 当前 `1/1` GREEN；snapshot 1354 已固定 exact11，首次独立复审为 `Critical 0 / Important 1 / Minor 0`，唯一 Important 即本记录的 stale r1/exact5/API 描述。
- Frameworks01 scene no-default-features job `d2bad3c6a3dc40d5860f11d1400003e9` 提供了 focused RED：kernel test 通过私有 `render::environment` 导入触发 E0603；公开 `framework::render` facade 已导出所需符号，最低修复不需要开放内部 module。
- kernel test 已改为从既有公开 `crate::core::framework::render` facade 导入，不公开 `environment` 内部模块、不新增兼容 re-export；source-cubemap 子树和 kernel test 叶文件已通过 Rust 1.94.1 scoped rustfmt。
- 最终静态证据为 boundary guard `1/1` GREEN、scoped rustfmt check GREEN、dirty exact9 `git diff --check` GREEN；snapshot 1354 已因本记录和 import/format 变化作废，不得复用。
- r2 的旧 wrapper 把 11 个路径错误登记为一个分号拼接且不可变的 `write_scope` 项。r2 已在保留源码和 review evidence 的前提下取消并释放 11 个租约；`frameworks01-m1-contracts-kernel-test-boundary-r3-20260731` 使用“父计划 + 编号子目录 + exact11”数组 scope 重新领取和 attribution 同一组哈希，不使用直接数据库修补或隐式 scope 扩张。
- snapshot 1356 对最终代码/文档内容的独立复审为 `Critical 0 / Important 0 / Minor 0`；r3 snapshot 1357 与 1356 的 11 个内容哈希逐项一致。本文状态更新后必须再生成 fresh r3 snapshot，1356/1357 都不得直接绑定 managed gate。
- M1 milestone prepare 的首次 wrapper 调用仅生成 action `37a6a05fba99442a9e9de7062d8668bd` 的未确认 preview，已自然过期且未生成 manifest；第二次受控 action `300a735047944a479dd954f680f2ce33` 明确失败为 `milestone_manifest_record_ambiguous`，原因是本文 `Plan:` 值包含 Markdown 反引号而不匹配 Session 的 canonical `plan_path`。现已按精确协议移除反引号；两次动作均未启动 validation/Cargo，不作为 managed gate 证据。
- 修正 `Plan:` 后的第三次受控 action `caa1a2f8feea42ca975464d8dae1abb9` 明确失败为 `milestone_manifest_not_attributed`：治理 scope 中的 `mipmap.rs` 与 `pmrem_layout.rs` 当前字节和 HEAD 完全一致，不是 dirty change。当前 milestone `Files:` 因此收敛为真实 dirty exact9；两文件仍由 r3 scope/lease 保护并由 baseline 自动进入 validation-copy，不制造空改动、不伪造提交归属。

### 待完成项目

- 刷新本文更新后的 immutable dirty exact9 snapshot/attribution，复核独立 `C0/I0/M0` 结论并执行 fresh validation-copy focused Rust gate；旧 materialization 和 snapshot 1354/1356/1357/1360/1361 均不得在源变更后复用。
- managed Rust GREEN 后通过 coordinator 原子提交，并把真实 job/run/结果写回本记录；在此之前不声明 Rust gate 通过。
- 完成后刷新 Frameworks01 M0/M1 状态；在此之前不声明 `zr_contracts` 创建前置完成，也不开始物理 crate move。
