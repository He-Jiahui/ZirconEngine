---
related_code:
  - zircon_runtime_interface/src/math.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
tests:
  - zircon_runtime_interface/src/tests/contracts.rs
  - zircon_runtime/tests/math_precision_contract.rs
  - current-source Windows math tests pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime interface math 性能静态审查（2026-07-23）

## 范围与覆盖

`zircon_runtime_interface/src/math.rs` 当前源 **1/1** 个 Rust 文件、**193** 行已逐行阅读，并反查 runtime fallback、shadow validation和Editor viewport等生产消费者。接口合同1条、runtime precision合同5条覆盖f32 ABI、TRS、parent/local顺序、affine inverse、finite与render conversion语义。文件当前无工作区改动，本轮未修改源码。

## 性能结论

- `Transform`、compose/inverse/view/projection与finite helpers均直接调用glam值类型操作，无堆分配、锁、I/O、全局扫描或任务调度；没有形成独立的主线程规模瓶颈。
- 当前`Real`与`RenderScalar`都是`f32`，source/render `Vec*`和`Mat4`也为同一glam类型；`to_render_vec2/3/4`仍逐分量调用`to_render_scalar`，`to_render_mat4`先导出16元素数组、执行16次分支后重建矩阵。它是可测的常数级微成本，但现有consumer没有证明它支配帧预算，因此不新增PERF-MVP编号。
- 后续若F2/F4 trace把conversion列为热点，可在不改变None-on-non-finite合同下收敛为一次vector/matrix finite检查后原值返回；必须以call count、branch和frame p95证明收益，并保持NaN/Infinity负例及f32 ABI。未经该动态证据，不以微优化增加另一条math实现。
- `looking_at`的normalize/cross与matrix-to-quaternion只在显式transform构造时执行；`perspective`仅做常数次参数clamp。当前无每帧重复构造或错误算法证据。

## 动态验收

1. current-source interface math合同与runtime precision 5条测试全部通过，finite/NaN/Infinity、TRS、inverse和parent/local顺序不变。
2. F2场景与F4 viewport记录math helper call count、conversion占用、branch和CPU p95；只在conversion进入top hotspot时实施single finite-check fast path。
3. 1/10k/100k transform批次记录allocations=0、bytes copied和吞吐；任何优化保持`Real`/`RenderScalar` 4-byte ABI及相同矩阵结果容差。

current-source Cargo、transform规模counter与F2/F4产品trace未完成，因此该文件继续保留在 `pending.md`，不进入 `review.md`。
