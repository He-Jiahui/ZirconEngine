---
related_code:
  - zircon_app/src/entry/tests
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
tests:
  - current tree contains 75 #[test] functions
  - current-source managed Windows Cargo pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# App entry tests当前源码复核（2026-07-23）

## 范围与基线

`zircon_app/src/entry/tests/**`当前源码 **42/42** 个Rust文件、**4,380** 行、**75** 条测试已逐文件阅读；path+Git-blob清单SHA-256为`44de3f4982df4ec25922aa83490e4322cad8178a9a68a8000defb89367f758c6`。9个tracked测试文件有外部未提交修改，本轮只读保留。

## 覆盖与缺口

测试覆盖profile/module/plugin选择、native/export bootstrap、window/input/IME/gamepad ABI顺序、surface fallback、window lifecycle和目录owner。它们以短行为测试与`include_str`源码锚为主，能防结构/调用顺序回退，但没有F0性能验收：

- 没有dynamic descriptor repeated construction/free counter，当前测试多次构造linked module却不检测`Box::leak`或RSS回落；PERF-MVP-004仍open。
- 没有registration/catalog/config/project-open次数，不能验收PERF-MVP-427。
- 没有30秒idle、unhandled-event、single-control-flow、same-size resize、host/input storm、gilrs wake、owned-output free或failed-destroy counter，不能验收023/424..426/574。
- `entry_tree`只断言一组已知路径存在，不比较完整文件集合/计数，也没有覆盖所有current新增leaf；它不能代替146文件current-hash账本。

部分source helpers为测试拼接多个完整源码String，`native_bootstrap_moves_owned_registration_reports`还`split_whitespace().collect::<String>()`；当前输入只有数KiB且不进产品路径，不建立MVP性能编号。若结构测试继续扩大，Runtime02/zircon_app应改为逐文件token/AST或共享一次source fixture，但优先级低于产品动态门。

current-source managed Cargo尚未运行，199条全入口`#[test]`也只是源码计数而非通过数；本目录继续留在`pending.md`，不进入`review.md`。
