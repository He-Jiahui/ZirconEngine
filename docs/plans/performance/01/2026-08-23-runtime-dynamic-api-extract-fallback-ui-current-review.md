---
related_code:
  - zircon_runtime/src/dynamic_api/session/extract.rs
  - zircon_runtime/src/dynamic_api/session/extract_cache.rs
  - zircon_runtime/src/dynamic_api/session/extract_stats.rs
  - zircon_runtime/src/dynamic_api/session/menu.rs
  - zircon_runtime/src/dynamic_api/session/hud.rs
  - zircon_runtime/src/dynamic_api/session/preview.rs
  - zircon_runtime/src/dynamic_api/session/highlight_set.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/01/2026-08-14-runtime-scene-render-extract-current-review.md
  - docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
  - docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/ScenePrivate.h
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/RendererScene.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/GPUScene.cpp
tests:
  - current extract and fallback UI 7 of 7 Rust files and 8 tests reviewed
  - M0 static performance contract 3 of 3 passed
  - focused rustfmt plus 1.94.1 and scoped diff check passed
  - current-source Cargo pending
doc_type: implementation-evidence
status: m0_static_complete_dynamic_blocked
---

# Runtime dynamic API extract/fallback UI当前复审（2026-08-23）

## 范围与当前性

已逐行复读extract/cache/stats与menu/HUD/preview/highlight共 **7/7** 个Rust文件、**1,180行、41,058 B、8 tests**，manifest SHA256为`b27bf46cefec8a0684c27c868120db87f5010e6479c173a83d6f3c2ab28473ad`；当前均干净。并与2026-08-14的scene render-extract纵向报告核对，旧PERF-MVP-431/433需要按current source修正而非照抄。

## current source结论

- cache命中仍执行`entry.extract.clone()`，miss也为cache entry执行一次完整clone；diagnostic明确把两种路径都计为`full_clones=1`。graphics提交内部已经使用`Arc<RenderFrameExtract>`，dynamic API入口仍在Arc owner之前复制宽DTO。
- cache key使用global World change tick、lifecycle visibility revision、active camera和viewport。任意与render无关的World write都会失效完整extract；LevelSystem animation publication又不在该key中。PERF-MVP-620的统一producer/domain generation仍是P0。
- `RuntimeFrameExtractDiagnosticsSummary`已经在cache miss与extract同时封存，稳定hit不再扫描mesh/VG/light/post/UI等宽payload。旧报告“每capture/present全扫估算bytes”已过时；剩余hit成本是deep clone和7条diagnostic record。
- 每次capture/present仍逐条调用7次`CoreRuntime::record_diagnostic`：7次diagnostics mutex、7次BTree path lookup，并走通用unit/tag metadata入口。路径、unit和tags均为静态常量，当前`DiagnosticStore::record_static`已有metadata匹配快路，因此可以安全合并为一次锁。
- fallback menu/HUD已从全node scan改为dynamic component sparse index，HUD token分类无临时Vec，menu hit-test不构造presentation Strings。这些M0已生效；但每次fallback present仍重查menu/HUD generation，存在menu时重建6 commands及颜色/文本Strings，存在HUD时重建2 commands。PERF-MVP-433继续要求component generation-owned UI extract。
- `dynamic_component_rows`对该component全部owner收集并按entity排序，menu只取第一项；HUD对两个component分别收集并排序。典型单owner规模小，不能在没有产品规模证据时优先改scene API；最终UI generation owner应直接消费component delta/index。

## M0与结构计划

M0仅修改`extract_stats.rs`：取得一次Core handle/diagnostic store短锁，在同一closure内用`record_static`写7条指标。目标是每frame diagnostics lock acquisition `7 -> 1`、通用metadata path `7 -> 0`；指标名称、frame index、history和数值完全不变。先加source contract RED，再实施并保留现有frame diagnostics行为测试。

结构任务不变：PERF-MVP-620/349发布唯一LevelSystem/render-world scene generation，cache与graphics传`Arc`/generation handle，稳定hit full clone=0；PERF-MVP-433按menu/HUD component generation+viewport generation发布唯一UI extract。Unreal依据沿用本地`ScenePrivate.h:1561-1596`的persistent packed scene arrays、`RendererScene.cpp:1570-1709`的changed primitive enqueue/batch update和`GPUScene.cpp:1821-1842`的dirty persistent primitive publication：先有持久scene owner，再按change更新，不从gameplay World每帧clone完整DTO。

## 验收状态

M0静态门要求1个batch update closure、7个`record_static`、函数内`runtime.record_diagnostic`为0；动态按diagnostics enabled/disabled、stable/miss、60/120/240 Hz记录lock acquisitions/wait/hold、path/tag/unit allocations和record p95。结构门按0/1/1k/100k entities/meshes、0/64 MiB sideband、stable/selection/UI/animation/1% change记录World clone、extract build/full clone bytes、UI sparse visits/build/String alloc和F2/F4 frame CPU/GPU/energy。current Cargo/WPR/RenderDoc未通过前留在`pending.md`，不进入`review.md`。

## 2026-08-23 M0实施证据

`tools/tests/test_runtime_extract_diagnostics_m0_performance_contract.py`先得到3/3 failures RED，实施后3/3 GREEN；38行、1,177 B、SHA256 `0f6ca8dfee21b822a7491380669e499af1142a67a8a66b254251b9f3b2265373`。`RuntimeFrameExtractStats::record_diagnostics`现在只调用一次`update_diagnostic_store`，同一短锁内执行7次`record_static`；静态lock入口`7 -> 1`、通用metadata入口`7 -> 0`。focused `rustfmt +1.94.1 --edition 2021 --check`与scoped diff check通过。

实施后本切片为7文件、1,183行、41,364 B、8 tests，manifest SHA256 `93822c00e41a34838703c30424561cf23e632ef6120e0758fe2f49551437ae4a`。受管current Cargo不可执行，已有frame diagnostics Rust行为测试未运行；没有wall-clock、lock contention或功耗实测，因此只声明静态操作收敛。
