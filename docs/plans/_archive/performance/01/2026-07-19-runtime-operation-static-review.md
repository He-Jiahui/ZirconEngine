---
related_code:
  - zircon_runtime/src/operation
  - zircon_runtime/src/dynamic_api/session/operation.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
reference_sources:
  - dev/bevy/crates/bevy_tasks/src/task_pool.rs
  - dev/godot/core/object/worker_thread_pool.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Async/TaskGraphInterfaces.h
tests:
  - zircon_runtime/src/operation/tests.rs
  - current-source Windows Cargo and operation-storm product traces pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime operation逐文件性能静态审查（2026-07-19）

## 范围与结论

`zircon_runtime/src/operation/**`当前源 **7/7** 个Rust文件、**447** 行、**3** 条测试已逐文件阅读，覆盖handler/context/error、task state、submit/poll/harvest与基础行为。

`RuntimeOperationService`虽声明asynchronous，第一次poll只切到Running，第二次poll却在caller上同步执行整个handler并持有`&mut World`；dynamic API调用方同时持session mutex。queued/running/completed task及payload/result没有capacity、bytes、deadline、cancel或TTL，未harvest即可无界驻留。handler panic由外层FFI捕获时，task已置`execution_started=true`但不会转terminal，后续永久Running。每次progress还创建message String，submit/poll/harvest跨ABI重复JSON。

## 本轮直接止损

terminal harvest改用`HashMap::entry`在一次probe内验证terminal并remove，删除原`get + remove`双查找；UnknownHandle/NotTerminal及成功语义不变。RED→GREEN源码守卫、`rustfmt`与`git diff --check`通过。

## 参考与计划

Bevy TaskPool把task自动驱动在受控worker上；Godot WorkerThreadPool区分priority、queue与显式completion wait；UE TaskGraph显式选择named/any thread并将wait作为可见同步点。Zircon需要复用Runtime11统一budget/cancel/age，而不是每operation私建线程；World mutation保留有预算owner-thread apply。详见PERF-MVP-435与Runtime11 failure记录。

## 动态验收

1/1k/100k operations、0/10/1000ms handler、0/1/64MiB payload/result及harvest/no-harvest/panic/cancel组合记录caller work、session hold、queue/task/result bytes、age/drop/cancel和p95。current-source Cargo与F4真实操作trace完成前留在`pending.md`。
