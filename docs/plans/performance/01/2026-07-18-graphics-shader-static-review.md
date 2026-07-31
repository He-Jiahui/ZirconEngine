---
related_code:
  - zircon_runtime/src/graphics/shader
tests:
  - graphics shader current source 23 of 23 Rust files and 7209 lines reviewed
  - all 67 tests read; two source and ownership regressions added
  - preview batch indexing and include manifest ownership changed from RED to GREEN
  - scoped rustfmt, source contracts and diff check passed
  - current-source Cargo, F2 cache/compile counters, IDE scale trace and RenderDoc pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Graphics shader静态审查（2026-07-18）

## 当前源覆盖

`graphics/shader/**`当前23/23个Rust文件、7,209行已逐文件静态阅读，67条测试已读：root 7/7、`ide_env_generation/**` 1/1、`template/**` 11/11、`variant_cache/**` 4/4。覆盖固定compute/fullscreen contracts、WGPU layout投影、IDE stub/preview、WGSL template/include DAG、Naga validation、disk cache与prewarm全链。

## 直接止损

IDE preview matrix原对每个surface shader×preview variant调用公开单项API，单项API又对同一批全部shader URI重建HashMap。现批次入口先建一次index并传入内部assembly；独立公开API仍为单项caller自行建一次，语义不变。源码守卫把matrix的index build从S×V降为1/batch。

forward/deferred/TAA三套template assembly结束时原分别遍历include registry两次，把全部token与content hash String clone成输出Vec。registry此后立即丢弃，本轮改为`into_manifest(self)`一次遍历、按已知长度预留并移动token/hash，六类clone投影降为0；WGSL、include顺序、hash、segments和现有67条语义测试合同不变。

## 剩余根因

PERF-MVP-356继续负责frame线程首次variant：disk hit同步exists/meta+WGSL read、JSON、zstd decode，miss同步Naga/assembly、zstd/pretty JSON、双atomic write和driver module/pipeline创建。磁盘key未完整吸收template/Naga/WGPU version，不能在prewarm中简单把existing entry当作可信skip；须由typed queued state、content-addressed key与last-good fallback共同处理。

PERF-MVP-357继续负责prewarm：manifest仍按variant拥有完整WGSL/include hashes/version，执行器串行对每项做Naga、可选WGPU module/pipeline validation、压缩和写盘，report又逐variant构造dimension/provenance；没有worker/in-flight/RSS预算，也没有同source共享正文。

PERF-MVP-358现扩展到IDE env与template registry。每次assembly都重建约19个builtin includes，重新extract/strip/hash大WGSL并把完整source克隆进module resolution/output；IDE env每次加载全部ready shader，对每个stub递归时反复全扫stubs，拼接完整依赖源码后逐stub Naga parse，即使第二次生成零文件写入也仍parse全部module。preview为S×V完整assembly+Naga validate，stale cleanup递归扫输出树。需要按source/module generation持久化单遍parse artifact、indexed dependency DAG、content-addressed builtin modules和增量preview artifact。

本地Bevy `pipeline_cache.rs`使用`CachedPipelineId`与waiting queue延迟GPU创建；UE `ShaderCompiler.cpp`/`ShaderCompilerJobCache.cpp`以本地/远程workers、异步DDC/job cache、memory budget和async result阶段分离编译。采用“稳定generation零重算、content key去重、有界后台工作、frame/UI线程只poll ticket”的原则，不复制其API或进程模型。

## 验收状态

23/23静态阅读、两条RED→GREEN回归、rustfmt、源码合同与diff门禁完成。Windows Cargo validator仍在启动前`ConvertFrom-Json`失败，67条测试没有current-source执行结果；RenderDoc CLI不可用。sources/stubs/variants 1/100/10k/100k、WGSL 4KiB/1MiB、include depth 1/100/1k、stable/1% change、workers 1/8/64下的index/DAG builds、source scans/hash bytes、assembly/Naga/zstd/I/O、queue age/RSS、frame/UI stall、pipeline miss及GPU timestamp未量化，继续留在`pending.md`，不进入`review.md`。
