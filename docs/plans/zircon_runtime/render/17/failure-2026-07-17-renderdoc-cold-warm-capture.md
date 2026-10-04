---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: renderdoc-cold-warm-capture
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/render/17-performance-and-profiling.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/render/17
plan_link_mode: child_record_only
related_code:
  - docs/plans/performance/01/renderdoc_capture_audit.py
  - tools/tests/test_renderdoc_capture_audit.py
tests:
  - python -B -m unittest tools.tests.test_renderdoc_capture_audit -v
  - current-source MVP cold-frame RenderDoc capture
  - same-process second stable-frame RenderDoc capture
  - GPU timestamp availability and missing-counter reporting
---

# Render17：旧 D3D12 capture 显示 copy storm，但缺当前源码冷暖帧对照

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：RenderDoc 1.44 工具链探测与旧 D3D12 capture replay
- 来源证据：`docs/plans/performance/01/2026-07-17-renderdoc-toolchain-probe.md`
- 修复责任计划：`docs/plans/zircon_runtime/render/17-performance-and-profiling.md`
- 交接原因：冷/暖帧捕获规范、GPU counter 缺失语义和跨 render owner 路由属于 Render17。

## 失败现象与复现证据

RenderDoc 1.44 已成功 replay 现有 D3D12 capture：4,357 actions、58 draws、39 dispatch、3,506 copy、51 clears；3,203 copy 发生在 event 4,000 前，copy 占 actions 80.47%。GPU duration counter 可枚举但没有样本，不能写成 0 ms。

## 最低共享层根因

该 capture 是旧高级场景且 copy 高度集中于早期事件，当前只能判定为冷帧初始化候选，不能证明当前源码稳定帧存在上传风暴。

## 架构修复验收

- 用当前源码、同一进程、同一 MVP 场景捕获 cold frame 与至少第二个 stable frame。
- 报告 draw/dispatch/copy/clear/barrier、upload bytes、pipeline/资源创建和 GPU duration；缺失 counter 显式记 unavailable。
- 静态稳定帧上传应归零或每项有资源生命周期理由；若仍有 copy storm，再路由至 Render01/02/03/13 的最低 owner。

## 禁止临时方案

- 不得用旧 capture 冒充当前源码或稳定帧。
- 不得把缺失 GPU timing 写成零，也不得通过降低画质掩盖上传/同步问题。

## 修复结果与回传

Open state: `连续的ZR_RENDERDOC_CAPTURE_FRAME_COUNT=2只适合相邻帧，不足以代表该temporal full-chain产品路径的settled warm frame：第二帧会编译history-enabled图变体。ignored exporter export_render17_pfm1_render_graph_cold_warm_wgpu_png现先创建docs/tests/runtime/render，要求进程已注入RenderDoc，并在WGPU初始化前通过RenderDoc v1 API配置进程唯一的capture模板；它随后手动捕获cold帧、渲染history-transition帧、再手动捕获settled-warm帧，写入cold/warm两张PNG，要求两次capture stop均成功以及恰有两份匹配模板的.rdc，并额外写入同一对帧的JSON profile manifest。manifest把图缓存计数标为累计值，capture-frame profile与可能延迟的resolved GPU profile分开记录；RenderDoc draw/dispatch/copy与GPU event duration标为unavailable_pending_renderdoc_replay，不能替代RDC回放。二次独立静态审查已完成，未发现Critical、Important或Minor问题；其确认RenderDoc v1 ABI前缀、Windows调用约定、注入模块生命周期和进程级GetAPI互斥均正确。scene与retained UI现共享RHI WGPU timestamp owner，UI样本以Option和异步回读延迟上报，缺失样本不写成0；scene timer使用frame-profiler generation而非mesh-command cache generation，三槽异步回读按generation有序出队。上述是当前源码前向修复，不是验收：当前目录仍无本轮current-source PNG/RDC，且没有受控Cargo构建、draw/dispatch/copy/upload或GPU timing复盘。待这些实际证据完成后回传`。

2026-08-10 independent review continuation: the current evidence contract received `Critical 0 / Important 0`; the only Minor finding was unformatted owned Rust sources, repaired mechanically and rechecked with `rustfmt --check`. The reviewer separately confirmed injected-template ordering, cold/history-transition/settled-warm sequencing, the exact-two-RDC assertion, manifest provenance, and the boundary between direct presentation and explicit CPU capture. This does not change the `open` status: a managed current-source Cargo/WGPU run, two current PNGs, current RDC replay, and unavailable-or-measured draw/dispatch/copy/GPU timing are still required.

2026-08-27 audit-report continuation: `renderdoc_capture_audit.py` no longer
collapses an exposed duration counter with zero returned samples to `0 ms`.
The JSON report now distinguishes `unavailable_counter_not_exposed`,
`unavailable_no_samples`, and `available`, includes the sample count, and emits
`gpu_duration_total_ms: null` for both unavailable states. Fake-controller
regressions pass 3/3 and cover both unavailable cases plus measured ordering and
total duration. This removes one evidence-integrity defect but does not provide
the missing current-source cold/settled-warm captures or replay metrics, so the
failure remains `open`.

### 2026-09-19 rolling validation admission

Successor Session `failure-roll-01a084c8-render17-renderdoc-cold-warm-r1` claimed
the failure record, audit script, and audit regression under the coordinator at
baseline epoch `611`. The exact current-source hashes are:

- `docs/plans/zircon_runtime/render/17/failure-2026-07-17-renderdoc-cold-warm-capture.md`:
  `877b40a2e150fee597ed99135e4d529aff04acca7c2e813cc417216bca53d0ac`;
- `docs/plans/performance/01/renderdoc_capture_audit.py`:
  `76dbb6bce9e383553ebc5a9b085502f944905363e4e468f79241e15187a3e1a8`;
- `tools/tests/test_renderdoc_capture_audit.py`:
  `aa6e589231d25edb280e637f44783c307d881b6c04afda31523bd2e5e662f03c`.

`python -B -m py_compile docs/plans/performance/01/renderdoc_capture_audit.py`
passed, and the exact regression command
`python -B -m unittest tools.tests.test_renderdoc_capture_audit -v` executed
all `3/3` tests with exit `0`. A source-contract probe also confirmed that both
unavailable GPU-duration states retain `total_ms: null` and the measured state
remains explicit. This is lower audit evidence only: no current-source Cargo
run, PNG/RDC pair, RenderDoc replay, draw/dispatch/copy report, upload-byte
measurement, or GPU timing result was produced. The lifecycle therefore remains
`open` and is not eligible for fixed return or closeout.

The coordinator accepted static audit ticket `15773b0d10864c398a11f1f8473f8b6c`
for this exact three-file manifest (`source_manifest_hash`
`8f613e77d60f3177beb3db973862c63bfdb728bba50a4b8de1aa6d6d0dbf6e94`). Its
command runs the three-test audit regression and `py_compile` on Windows; the
ticket deliberately declares `upwardAcceptance: false` and defers all current
source Cargo/WGPU and RenderDoc capture/replay evidence. It is queued for the
coordinator worker and cannot be reused as a product acceptance result.

The static audit ticket completed on managed job/run
`f28436a1784d4189b15ee85e63616736` with exit code 0. Ticket
`15773b0d10864c398a11f1f8473f8b6c` emitted the three-test receipt (including
the measured and unavailable GPU-duration cases) and cleanup completed. This
does not provide a PNG/RDC pair, RenderDoc replay, WGPU/Cargo run or GPU timing
evidence; those gates, independent review, fixed return and closeout remain
pending.

### 2026-09-19 duplicate static audit receipt

The same fixing Session `failure-roll-01a084c8-render17-renderdoc-cold-warm-r1`
also retained ticket `f639349d70b14eeab4bd2783fdc55ce1` as an independent
coordinator run. Managed copy job
`6b040e6d0ca34e2b9e50a33d6e3a4298` and run
`f639349d70b14eeab4bd2783fdc55ce1` exited 0; the three audit regressions
(`test_gpu_duration_reports_counter_not_exposed`,
`test_gpu_duration_reports_exposed_counter_without_samples`, and
`test_gpu_duration_reports_measured_samples`) all passed. This is a duplicate
static audit receipt, not product acceptance: current-source Cargo/WGPU,
cold/history-transition/settled-warm PNG and RDC capture, replay metrics,
independent zero-finding review, canonical `failure return`, and closeout
remain pending.

### 2026-09-20 independent source review r1

Reviewer session `review-render17-renderdoc-cold-warm-r1` completed a read-only
review of the current audit implementation with `Critical=0 / Important=0 /
Moderate=0`. The review checked the recursive action-tree walk and event map,
copy-source/destination attribution, draw/dispatch/copy/clear/present counts,
counter enumeration, and the explicit GPU-duration states
`unavailable_counter_not_exposed`, `unavailable_no_samples`, and `available`.
Unavailable states preserve `sample_count=0`, `top_25=[]`, and
`gpu_duration_total_ms=null`; measured samples are sorted by duration and
aggregated without converting missing values into a fabricated zero. The
current exact commands also passed:

```text
python -B -m py_compile docs/plans/performance/01/renderdoc_capture_audit.py
python -B -m unittest tools.tests.test_renderdoc_capture_audit -v   # 3/3
```

The reviewed source hashes were:

```text
docs/plans/performance/01/renderdoc_capture_audit.py  76dbb6bce9e383553ebc5a9b085502f944905363e4e468f79241e15187a3e1a8
tools/tests/test_renderdoc_capture_audit.py            aa6e589231d25edb280e637f44783c307d881b6c04afda31523bd2e5e662f03c
```

This receipt is limited to the audit layer. It does not claim a current-source
RenderDoc cold/history-transition/settled-warm capture, PNG/RDC pair, replay
metrics, Cargo/WGPU result, or performance threshold; those gates, canonical
`fixed-*` return, and closeout remain pending.
