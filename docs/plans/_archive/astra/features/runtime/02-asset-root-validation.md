---
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/04/2026-09-09-manifest-bound-and-root-admission.md
  - docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md
  - docs/plans/optimize/zircon_runtime/161-runtime-core-resource-asset-serialization-load-artifact-pack-persistence-current-source-review.md
  - docs/plans/astra/performance/01-bounded-hotpaths.md
---

# 项目根目录与 manifest 边界优化

## 已确认问题

项目 manifest 加载调用 `ProjectManifest::validate`。重复 asset root 已采用 hash，
但重叠检查仍遍历所有目录对：10k 个互不重叠根目录需检查 49,995,000 对。
UI root 校验则为每项构造完整 URI 字符串，只用于临时集合去重。
Manifest 文件加载此前可按文件元数据无界增长，且运行时根目录准入没有显式复用
interface 的最大数量限制。

## 实施与验收

- asset root 在原重复检查阶段建立借用字符串到输入序号的索引；只查路径分隔符处的
  真正祖先，保留原始输入顺序决定的首个冲突和 duplicate-before-overlap 优先级。
- UI root 在 scheme、empty、label 检查之后借用规范化 path 去重；只在错误时格式化 URI。
- 用旧算法作为测试 oracle，覆盖目录排列、重复、边界前缀、规范化路径、空输入及 UI 错误优先级。
- release 同进程验证 1/1k/4096 个有效 asset root，并验证 1/1k/10k 个 UI root；预热后
  采样 p50/p95/p99。正常小输入 p95 回退不超过 5%，1k/4096 asset root 的 p95 目标
  为旧算法的 20% 以内，UI root 为 80% 以内。前后基准均调用完整 manifest 校验，
  包括版本和 template receipt 检查。10k asset root 仅可作为旧算法压力模型，不属于
  当前产品准入输入。
  性能结论须以实测为准。
- 本批同时交付两项修复，不逐项编译；与前一任务批次一起重提协调器。

## 计划完成列表

| 批次 | 内容 | 状态 | 验证证据 |
|---|---|---|---|
| M2 | 资产根目录祖先索引、UI 根目录借用去重、manifest 有界加载与 4096 根准入 | implemented_pending_validation | Runtime85/Astra 相关静态合同批量 `7/7`，manifest 生产/测试 Rustfmt 与 `py_compile` 通过；当前 Runtime02/08/19 + Editor09 聚焦源合同批次 `39/39` 已通过。Runtime85 释放基准仍是 helper microbenchmark，外部 `E:/Git/zr_vm` dirty，managed Cargo/release p50/p95/p99 待异步验收 |

## 复核修正

- 2026-09-05：修正祖先出现在后面的输入时错误字段颠倒问题，使用原始索引对只比较
  冲突优先级，返回值保留祖先/后代的语义顺序；720 种排列与旧实现逐项比较。
- 删除重复索引和每项临时集合，重复检查及祖先检查共用一个借用索引；单 asset root
  不创建索引。UI 去重仍保留 scheme/label 错误优先级。
- 修正测试中无法由类型解析器构造的空 URI，补充 UI 1/1k/10k 性能场景。
- 2026-09-09：加载端在 TOML 解析前按 `MAX_PROJECT_MANIFEST_BYTES` 读取 limit+1，字符串入口按 UTF-8 字节数拒绝超限；运行时 asset-root 数量与 interface 的 `MAX_PROJECT_ASSET_ROOTS = 4096` 收敛。

## 当前验证边界与源码指纹

本节覆盖 2026-09-10 的当前工作树，且 supersede 下方历史交接中的旧哈希与旧批次描述。
Runtime85/Astra 静态合同 `7/7` 通过，六个显式 Python 合同模块 `py_compile` 通过，相关
manifest owner 的 Rustfmt/解析与 scoped `git diff --check` 通过。未运行 Cargo，也未查询、
轮询或等待协调器；外部 `E:/Git/zr_vm` dirty 使 managed Windows/release 证据保持 pending。

Canonical 记录：[Runtime04 manifest bound and root admission](../../../optimize/zircon_runtime/04/2026-09-09-manifest-bound-and-root-admission.md)。

| 当前 owner | SHA-256 |
|---|---|
| `zircon_runtime/src/asset/project/manifest/load.rs` | `124203102FB84F9829C687128FCA4BC8943CD6A6E289A414FE5A54926FAEAF5B` |
| `zircon_runtime/src/asset/project/manifest/project_manifest.rs` | `862ECCBA6CA7F434D371065C86290F69C6AEFFA613F2AA2B824AEFFDAFEC7464` |
| `zircon_runtime/src/asset/project/manifest/validation.rs` | `1CC8C8552D43D4F2237DF92C70B871DCDDBD6209C1F25417432A1DC5558AE6B3` |
| `zircon_runtime/src/asset/project/manifest/error.rs` | `D394E4AB51BAA584AAF75B89B5055E494CD48B211B9F7A762ABA205FE9C9C727` |
| `zircon_runtime/src/asset/project/manifest/save.rs` | `8A251474650C8ADD87FD927E13F567FBB29E2DE947B59CC6F1C86617DAA85967` |
| `zircon_runtime/src/asset/project/manifest/validation/astra_root_tests.rs` | `1D9CEF99AD2AB4AC3DFF1D867331003D10645DBBD59E2D914CCF885B6D474EA1` |

The ignored release harness must still exercise real `ProjectManifest::validate` at 1/1k/4096
asset roots and 1/1k/10k UI roots before this row can become performance-qualified.

## 本切片静态复核（2026-09-18）

当前切片未再引入生产代码修改；限定 manifest 子树已完成 `git diff --check` 与 Rust
2021 `rustfmt --check`（production、focused regression、save/load tests）复核。静态
合同确认祖先索引只在路径分隔符边界查找，重复优先于重叠，UI root 在 scheme/empty/
label 校验后按借用规范化 path 去重；manifest load/save 的字节上限与 4096 asset-root
准入测试仍保持在同一 owner 子树。未运行 Cargo，性能证据仍按上文保持 pending。

本地当前 owner 指纹：

| 当前 owner | SHA-256 |
|---|---|
| `zircon_runtime/src/asset/project/manifest/error.rs` | `D394E4AB51BAA584AAF75B89B5055E494CD48B211B9F7A762ABA205FE9C9C727` |
| `zircon_runtime/src/asset/project/manifest/load.rs` | `3B406D030D29D2DB0E273822CCB1E620A842FEEF147DE061F3D1EE9451EFA841` |
| `zircon_runtime/src/asset/project/manifest/project_manifest.rs` | `862ECCBA6CA7F434D371065C86290F69C6AEFFA613F2AA2B824AEFFDAFEC7464` |
| `zircon_runtime/src/asset/project/manifest/save.rs` | `8A251474650C8ADD87FD927E13F567FBB29E2DE947B59CC6F1C86617DAA85967` |
| `zircon_runtime/src/asset/project/manifest/save/borrowed_serialization_tests.rs` | `A392BD1868FB99E04D77BE594FA18DA7309D2D4A87656DE411786B458A7CB9EF` |
| `zircon_runtime/src/asset/project/manifest/validation.rs` | `1CC8C8552D43D4F2237DF92C70B871DCDDBD6209C1F25417432A1DC5558AE6B3` |
| `zircon_runtime/src/asset/project/manifest/validation/astra_root_tests.rs` | `1D9CEF99AD2AB4AC3DFF1D867331003D10645DBBD59E2D914CCF885B6D474EA1` |
| `zircon_runtime/src/asset/project/manifest/validation/optimization_batch_ir_runtime629_tests.rs` | `C59FA854F39E27F6A7872B4D69DF0C826102D1C78920567E680CD52955914010` |

## 合并批次诊断交接

2026-09-05：M1 runtime panic、editor 进度派发及 M2 asset/UI root 一起通过协调器
现有 Cargo runner 提交。job `f95a4c5f25e549a49c21bec5bb43b694`，run
`777f5a8858494775841d089083550565`；提交回执为 `running`，不轮询编译状态。

命令：`cargo test --locked --release --no-default-features -p zircon_runtime -p zircon_editor --lib astra_m -- --include-ignored --nocapture --test-threads=1`。

日志目录：`.codex/state/session-coordinator/cargo-runs/f95a4c5f25e549a49c21bec5bb43b694/777f5a8858494775841d089083550565/`，分别为 `stdout.log` / `stderr.log`。
target 由协调器管理，位于 `E:/cargo-targets/zircon-engine/pool/3667796b29a891cd5c17310a54ea446a4714e097c686421df50fc894315712ef`。

这是共享当前工作树的诊断运行，外部 dirty 依赖仍未获得不可变快照验收；结果不得替代
integration validation ticket 或直接改写完成状态。提交后继续后续模块修复。

一次性读取诊断日志发现前置 E0277：host ProfileRootKeyVisitor 缺 Serde error 约束。
已取得租约补上约束，归属与验收见
[失败交接](../01/failure-2026-09-05-profile-root-visitor-error-bound.md)。本批尚无测试或性能通过数据。

| 输入文件 | 提交时 SHA-256 |
|---|---|
| `zircon_runtime/src/core/runtime/tasks/task_graph/scope.rs` | `6D088BDDA0E71B67DAA5AC7109AB85F6DA61EB939B482B4972CBB87B6296999B` |
| `zircon_runtime/src/core/runtime/tasks/task_graph/task_handle.rs` | `9C6CBF269A60E2CDAA1E4EC7FF33581DB85D1FFB0DF7BE524CD1D4B153B04DA8` |
| `zircon_runtime/src/core/runtime/tasks/task_graph/scope/tests/panic_payload.rs` | `0282B2AB5F7C9117E719DAA75C151BC5BED73A847EF3D6DC532F1471503BF48E` |
| `zircon_editor/src/core/jobs/system/progress_observer.rs` | `5832A08048A8F5B1ED9143B2EDC482DE7EA43FEB91A55D0FE86D3393F5F400A6` |
| `zircon_editor/src/core/jobs/system/progress_observer/reuse_tests.rs` | `CFEB8988236172B2B5117DA44B3AE2427981E834E0A651C786A21540BE616424` |
| `zircon_runtime/src/asset/project/manifest/validation.rs` | `8B4E146A8436F857BE3973943D05B0B786EC299A2CDFC099ECA335C7707D87E6` |
| `zircon_runtime/src/asset/project/manifest/validation/astra_root_tests.rs` | `7F95530C76F8BBD89371C245C9747C4986112106FD5D951B873A82B3105D29CD` |
| `zircon_runtime/src/asset/project/manifest/validation/optimization_batch_ir_runtime629_tests.rs` | `ACA80F61095A182A9BDB7B12875E12DEF78C1E488C876199B99C423D73258343` |
