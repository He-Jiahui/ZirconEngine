---
related_code:
  - docs/plans/performance/01/renderdoc_capture_audit.py
  - zircon_runtime/src/graphics
  - zircon_runtime/src/render_graph
implementation_files:
  - docs/plans/performance/01/renderdoc_capture_audit.py
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
tests:
  - renderdoccmd replay --loops 1 <capture>
  - qrenderdoc --python docs/plans/performance/01/renderdoc_capture_audit.py
doc_type: testing-guide
---

# RenderDoc v1.44 工具链探测与冷帧 copy 候选

## 范围

本次只验证 RenderDoc 自动回放/审计链路，并用现有的 2026-07-16 高级体积雾 DX12 capture 发现候选问题。该 capture 不是 F2 最小场景，不能作为 MVP draw、资源或 GPU 时间基线；正式 F2 仍需当前源码的冷启动帧与稳定第二帧成对捕获。

## 实际命令与结果

- `D:\Tools\renderdoc\renderdoccmd.exe --version`：v1.44，commit `050034a0faa37d606ce1b8cf677dba4bc36984ea`。
- `D:\Tools\renderdoc\qrenderdoc.exe --version`：v1.44，同一 commit。
- `renderdoccmd replay --loops 1 docs/tests/runtime/render/plan18_af_m3_volumetric_media_dx12_renderdoc_20260716_capture_capture.rdc`：exit 0；命令总耗时 18.7 s。
- QRenderDoc Python replay API 首次自动审计：exit 0；D3D12 capture 可打开、遍历和关闭，无 API debug message。

自动审计计数：

| metric | observed |
|---|---:|
| actions / max event id | 4,357 / 6,581 |
| draw / dispatch / clear / present | 58 / 39 / 51 / 1 |
| copy | 3,506（3,321 texture；185 buffer） |
| copy before event 4,000 | 3,203（91.36%） |
| command-list begin boundaries | 151 |
| resources / textures / buffers | 1,036 / 59 / 399 |
| API debug messages | 0 |
| GPU Duration samples | 0（counter 可枚举但 capture replay 未返回样本） |

## 解释与责任路由

copy 占全部 action 的 80.47%，且 91.36% 集中在 render graph 主 marker 之前的早期事件区。这更像首帧资源初始化/上传，而不是已经证明的稳定帧 pass 开销；现阶段不能从 action 数直接推导 GPU 瓶颈。

该证据回填 `docs/plans/zircon_runtime/render/17-performance-and-profiling.md` 的既有责任：

- `render_perf_upload_bytes_static_second_frame_zero` 必须用稳定第二帧证明静态场景 upload/copy 收敛为零或解释保留项。
- RenderDoc 标准化需要成对保留 cold-first-frame 与 warm-steady-frame capture，并附同帧 `RenderFrameProfile` / graph dump；不能用首帧 capture 代表 steady-state。
- GPU Duration counter 本次没有样本，GPU 毫秒必须来自计划 17 的 wgpu timestamp query 或确实返回 counter sample 的新 capture；不得写成 0 ms。

若 MVP 当前源码的稳定第二帧仍出现同量级 copy，本计划再按最低 owner 路由到 Render 03（增量上传）、13（纹理 staging）或 17（观测/预算），并创建正式 `failure-*` 交接。本轮不在尚未复现的情况下提前创建失败单。

## 可复用审计脚本

`renderdoc_capture_audit.py` 从环境变量读取：

- `ZR_RENDERDOC_CAPTURE`：输入 `.rdc`。
- `ZR_RENDERDOC_AUDIT_OUTPUT`：输出 JSON。

它只读 capture，输出 action 分类、copy 资源归因、marker、资源/API message 和可用 GPU duration；在已有 QRenderDoc UI 进程时应先确认命令没有被既有实例接管，输出文件时间必须晚于脚本执行时间。
