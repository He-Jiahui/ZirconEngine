---
related_code:
  - zircon_app/src/runtime_presenter.rs
  - zircon_app/src/tests
  - zircon_app/tests
  - zircon_app/src/entry/runtime_entry_app/surface_present
  - zircon_app/src/entry/runtime_library/runtime_session.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Slate/SceneViewport.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/SlateRHIRenderer/Private/SlateRHIRenderer.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/GameViewportClient.cpp
tests:
  - current-source hash stability 9/9 passed
  - direct rustfmt 9/9 passed
  - managed Windows Cargo and WPR/xperf/RenderDoc current-source matrix pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# App根目录、fallback presenter与测试current-source性能审查（2026-08-14）

## 范围与快照

`zircon_app`根/`src`余项 **7/7** 与`src/tests/**` **2/2**，合计 **9/9** 个Rust文件、**711** 行、**655** 个非空行、**8** 条`#[test]`已逐文件完整阅读；直接`rustfmt +1.94.1 --edition 2021 --check --config skip_children=true`为9/9通过，复核前后SHA-256前缀9/9不变。`tests/editor_mvp_authoring.rs`有其它会话修改，本轮只读、不覆盖。

| 文件组 | 行/非空行 | SHA-256前缀 | 结论 |
|---|---:|---|---|
| `build.rs`、`src/{lib,prelude}.rs` | 57/51 | `8AB25E965023`、`4C69815C7188`、`38111C01A733` | link配置与导出，无运行时热路径 |
| `src/runtime_presenter.rs` | 122/102 | `2A51CAA90F53` | CPU fallback逐像素RGBA -> XRGB，无临时Vec |
| `src/tests/{mod,prelude}.rs` | 106/100 | `F453B9D859FA`、`45D301A24F77` | prelude/多profile行为测试 |
| `tests/*.rs`三项 | 426/402 | `49659D4D843F`、`60F608FE8139`、`998A99E458BB` | process log源码guard、F4持久化E2E、typed plugin error |

8条测试中仅`diagnostic_log_process_lifecycle.rs`的1条通过`include_str!`读取两个bin源码并比较token顺序；其余7条调用真实API/文件流程。该源码guard不能证明panic/shutdown时真正flush，动态process test仍需补齐。

## 产品调用图与量化下界

```text
present_redraw_frame
  -> surface_present_enabled
       -> RuntimeSession::present_viewport                    native GPU present; success returns
  -> otherwise
       -> RuntimeSession::capture_frame                       owned RGBA CPU frame
       -> SoftbufferRuntimePresenter::present
            -> buffer_mut
            -> copy_rgba_to_xrgb                              scalar full-frame conversion
            -> buffer.present
```

当前源码先尝试native surface，成功后立即返回；只有没有native bind的fallback才请求完整`RuntimeFrame`。`RuntimeFrame`借用ABI owned buffer并在`Drop`归还，不在App再深拷贝；presenter复用Softbuffer surface，完整帧不预清，只有截断错误帧才先清零。该实现没有每帧临时Vec，但Softbuffer合同决定至少读取4 bytes/pixel并写入4 bytes/pixel：

`minimum local traffic = width * height * 8 * fps`

1080p/60最低约 **0.93 GiB/s**，4K/60最低 **3.71 GiB/s**；4K每帧RGBA输入与XRGB输出各31.64 MiB。此下界尚未计GPU readback、ABI buffer写入、surface map和OS present，所以不能用它冒充实测总带宽。现有`fallback_capture_request`、`fallback_rgba_bytes`、`fallback_cpu_present` counter已经提供验收锚点。

## 性能判断

### P0合同：native product path不得退化到CPU capture

默认F0/F2运行时必须保持native surface；稳定帧`fallback_capture_request/fallback_rgba_bytes/fallback_cpu_present`均为0，显式截图另计。该合同归Runtime10的动态ABI/surface owner与Render17的产品profile共同验收，并复用`PERF-MVP-023`对同步GPU -> CPU readback的统一治理，不另建重复根因。

### P1 fallback候选：全帧同步readback与颜色转换

forced fallback是兼容/诊断路径，但在native surface不可用的平台仍可能成为产品路径。先用WPR、GPU timestamp与RenderDoc拆分`capture_frame`同步等待、RGBA生产、转换和Softbuffer present；只有转换CPU占比显著时才评估SIMD/并行。手工向量化只改变常数，不能消除O(pixels)读写；结构优先级仍是恢复native surface，次选有界异步readback ring、明确帧率/分辨率降级或damage-aware present（仅当平台合同支持）。

### 不立项：导出与测试局部微优化

8 MiB MSVC stack reserve是链接时虚拟地址保留，不是每帧提交内存；`lib/prelude`只影响API与编译。F4 authoring E2E有意执行两次`EditorApplicationComposition::open_project`，再由`ProjectAuthority`第三次open/scan检查持久化，不是产品单次启动调用图，也不能直接作为性能benchmark。prelude测试重复构建多个plugin group只发生在测试，生产重复装配继续由`PERF-MVP-427`处理。

## 参考引擎依据

- Unreal `SceneViewport.cpp:2252-2324`把render target/back buffer交给render thread并保留buffered frame owner；`2327-2348`结束帧时只处理RHI resource/lifecycle，不把普通帧投影为CPU像素数组。
- Unreal `SlateRHIRenderer.cpp:1182-1188`把swap-chain/output texture转入`Present`，`1302-1339`在RHI viewport直接present并记录present latency；只有`screenshot`分支`1196-1221`增加readback pass与`ReadSurfaceData`。
- Unreal `GameViewportClient.cpp:2338-2552`只在movie/screenshot/high-res request成立时进入viewport screenshot读取和文件/委托处理。Zircon应保持同样的“native present是常态、CPU readback是显式证据/兼容边界”。

## 动态验收与跨计划交付

1. E/D/F盘current-source受管构建运行720p/1080p/4K x native/`ZR_RUNTIME_FORCE_CAPTURE_PRESENT` x 60/120 presented frames；WPR/xperf记录CPU sample、memory bandwidth/working set、CSwitch/ReadyThread、File I/O与energy，counter记录native/fallback次数和RGBA bytes。
2. native gate：稳定帧capture request/bytes/CPU present=0；RenderDoc显示present资源留在GPU，非显式截图无readback/copy-to-map；帧像素、resize、device loss与shutdown通过。
3. forced fallback gate：`rgba_bytes=width*height*4`每帧且presenter不创建临时full-frame Vec；分别报告capture/readback wait、conversion、buffer acquire与present p50/p95、峰值RSS、总带宽和energy。若达不到交互预算，优先恢复native合同或显式降级，不能只报scalar-loop microbenchmark。
4. 把process log源码形状guard补为真正启动bin/注入失败/等待退出/检查bounded flush顺序的行为测试；F4 E2E输出继续跟随E/D/F盘受管target，不落C盘，并单独记录create/open/import/automation/save/reopen阶段，禁止与产品warm startup混算。

| Owner计划 | 必须解决的合同 | Performance验收 |
|---|---|---|
| `zircon_runtime/runtime/10` | native bind/present与owned capture ABI互斥清晰；buffer release正确 | native帧capture=0；fallback每帧只保留一个owned RGBA generation |
| `zircon_runtime/render/17` | GPU timestamp/marker、readback fence、RenderDoc present/capture边界 | native无同步readback；fallback阶段p50/p95与GPU/CPU等待可归因 |
| App/Editor测试owner | process flush行为测试与F4阶段counter | 不再只凭源码token通过；E2E不冒充产品性能数据 |

本轮没有源码修改：当前实现已把重成本限制在fallback，且没有current-source产品profile支持局部转换优化。受管Cargo、native/forced-fallback WPR、GPU timestamp、RenderDoc和energy未完成，`zircon_app`余项与`src/tests/**`继续留在`pending.md`，不进入`review.md`。
