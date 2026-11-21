# 已完成的单个任务仍走集合等待路径

## 1. 单成员调用的开销

ZE 对 `TaskHandle::wait_all` 定义了三个分支。空输入直接返回，单成员使用该句柄自身的 `wait()`，多个成员才创建组合节点。公开实现对单个正常完成的任务先执行直接等待，随后又创建临时向量和组合节点。[快路径约定](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/docs/plans/_archive/astra/features/runtime/886-runtime02-task-handle-wait-all-fast-path.md#L19-L24) · [当前实现](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/core/runtime/tasks/task_graph/task_handle.rs#L83-L100)

## 2. 合法输入和可观察结果

使用 `EngineTaskGraph` 提交一个正常结束的工作，关闭该 scope 的准入并等待其完成。确认句柄状态为 `Completed` 后，分别用空切片、直接 `wait()` 和仅含该句柄的切片调用 API。调用线程的分配次数可以区分三种情况。这个问题影响公开 API 的单成员开销。当前已查到的动态场景调用使用单句柄的完成查询或直接等待。[真实句柄的使用位置](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/scene/dynamic/spawn_task/task.rs#L21-L35)

## 3. 多余的控制流

单成员分支中的 `handle.wait()` 返回后，控制流继续到 `collect::<Vec<_>>()` 和 `JobHandle::combine`。对这个非空输入，`combine` 分配一个新的任务节点，消费已结束的前置任务，并将组合节点设为完成。第二次调用 `wait()` 会直接看到终态。[组合实现](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/core/runtime/tasks/job_handle.rs#L134-L191) · [终态等待](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/core/runtime/tasks/job_handle.rs#L253-L284)

## 4. Unreal 源码与原有测试

UE 将直接等待委托给单个 `FTask`。集合重载以输入任务为前置条件创建一个 `Inline` 等待任务。两个重载位于 `Engine/Source/Runtime/Core/Public/Tasks/Task.h` 第 422–445 行。

注册原测 `System::Core::Tasks::Basic` 位于 `Engine/Source/Runtime/Core/Tests/Tasks/TasksTest.cpp` 第 30–36、91–109 行。本次按行为对应，不复制 UE 代码：

| UE 行为 | ZE 实现和测试 |
| --- | --- |
| 任务写入 `Done`，调用者先观察 `IsCompleted()`，再 `Wait()` 并检查 `Done`。 | ZE 在有界 scope 内等到句柄报告 `Completed`，分别做直接等待和单成员集合等待，再核对任务体通过原子量发布结果。见[分配与行为回归](../../../zircon_runtime/tests/runtime_task_node_allocation_performance.rs)。 |
| `Wait(Tasks)` 等待所有已提交任务。 | 多成员仍走 `JobHandle::combine`；[现有依赖用例](../../../zircon_runtime/src/core/runtime/tasks/task_graph/scope/tests/dependency_scale.rs#L50-L189)检查全部任务体已执行，以及出现 panic 时仍先等待所有成员终结。 |

**有意保留的差异：** UE 的集合重载创建 `Inline` 等待任务，然后用超时参数等待它并返回布尔值。ZE 使用自研 `EngineTaskGraph`、`TaskGraphScope` 和 Rust 任务句柄：`wait_all` 不带超时，正常完成和取消不通过返回值区分，任务 panic 在全部成员终结后传播。空切片直接返回，单成员复用原句柄而不创建组合节点，是 ZE 自有的快路径；零分配断言只针对已完成的成功任务、只统计调用线程，不是 UE 承诺。此修复不改变 ECS、变换、脚本虚拟机、模块裁剪或 WebGPU 路径。

## 5. 现有验证与历史归属

已有行为用例断言单成员等待后任务已完成。源码检查只查找分支文字，以及收集向量之前的一段代码。另一个性能标记直接写入预期的零个组合节点。这三项检查都不足以观察已经结束的单成员分支是否又进入后续路径。已核对所有状态 PR，任务句柄实现和相应的分配测试文件均无同题草稿。[现有测试](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/core/runtime/tasks/task_graph/scope/tests/dependency_scale.rs#L193-L247)

## 6. 定向回归与修复

扩展现有原生集成测试中的分配计数器，仅测调用线程。用一次明确的分配检查计数器正在工作，并核对空输入与直接等待的分配次数。对已经完成的同一个句柄，单成员集合等待应与直接等待相同。实现可将向量收集与组合等待放入多成员分支。[集成测试目标](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/tests/runtime_task_node_allocation_performance.rs#L14-L74)

## 7. 修复前的原生回归

同一已完成任务上，调用线程的单成员等待分配了 4 次，直接等待和空切片各分配 0 次。显式分配对照记录 1 次。

用例提交正常工作，观察到一个完成任务，随后执行三种等待。测试结束时检查任务状态并等待工作线程退出。

在 Windows x64 的 `test` 配置下，`zircon_runtime` 运行定向用例，通过 0/1。测试提交为 `65278a7c5ae8997f08a1b0e6819b527a4ae6bc22`。[本次回归用例](https://github.com/He-Jiahui/ZirconEngine/blob/65278a7c5ae8997f08a1b0e6819b527a4ae6bc22/zircon_runtime/tests/runtime_task_node_allocation_performance.rs)

## 8. 修复后的原生回归

多成员分支现在独占向量和组合节点。单成员等待的实测分配由 4 次降到 0 次。空输入和直接等待均为 0 次，显式分配对照为 1 次。

用例提交正常工作，观察到一个完成任务，随后执行三种等待。测试结束时检查任务状态并等待工作线程退出。

在 Windows x64 的 `test` 配置下，`zircon_runtime` 运行定向用例，通过 1/1。测试提交为 `39ddf7deaa559337fe1a5b92059c021ba6758fa1`。[本次回归用例](https://github.com/He-Jiahui/ZirconEngine/blob/39ddf7deaa559337fe1a5b92059c021ba6758fa1/zircon_runtime/tests/runtime_task_node_allocation_performance.rs)

## 9. 独立复核

以远端 `main` 的 `e89349ca5` 为合入基线，在 Windows x64/MSVC、Rust 1.94.1 进行复核。Cargo 使用锁定依赖、关闭默认特性，仅启用 `core-min`；通过规范仓库的独立本地入口在 PR 工作树执行。未运行 UE 测试二进制、跨平台矩阵、Release 基准或编辑器产品验收。

| 实际执行范围 | 结果 |
| --- | --- |
| `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --offline --jobs 1`（dev） | 退出码 0。 |
| `cargo test -p zircon_runtime --no-default-features --features core-min --locked --offline --jobs 1 --test runtime_task_node_allocation_performance task_wait_all_singleton_reuses_terminal_handle -- --exact --nocapture --test-threads 1`（test） | 1 通过；`empty=0 direct=0 singleton=0 meter=1`，任务体发布的结果保持可见。 |
| `cargo test -p zircon_runtime --lib --no-default-features --features core-min --locked --offline --jobs 1 task_graph::scope::tests::dependency_scale -- --test-threads 1`（test） | 5 通过、0 失败、1 个原有性能用例按标记忽略。覆盖空/单成员等待、依赖链、宽集合等待和全部终结后才传播 panic。 |

远端分支保护仍要求 GitHub Actions 的 `editor-compile-gate`。提供该门禁的 [CI 工作流](../../../.github/workflows/ci.yml#L37-L44)手动停用，未给出检查结果。本地范围通过不替代远端验收，不绕过门禁。
