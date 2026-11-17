# Core 内存诊断：成功 realloc 的两次计账让并发堆峰值漏计

> 取证说明：UE 的本机路径与文件 SHA256 是用户提供参考树的依据；本轮 D: 审计快照/工作记录仅本地保存，不随本 PR 发布，也不冒充 GitHub 可读证据。UE/ZE 测试均未在本轮执行。

**状态：独立 Draft 候选；源码证明一个可达交错，未观测产品症状。** ZE 基线 `e89349ca51b4e9d024c194cd3bdf645bd25b6c37`，本地 `cb495bb1c952a24c86f4513b8099bc4942c3a87d` 的相关代码无差异。UE 为 [本机 Build.version:2–9](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Build/Build.version#L2-L9>) 声明的 6.0.0、BranchName `UE5`、CL 0；源码提交未知。**未运行 Cargo、UE/ZE 测试、产品或 benchmark；未改引擎/索引/远端。**

## 1. 与 UE 不一致（两边可达路径）。

ZE 可选 [`zircon_app` `profiling-memory`:155–158](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_app/Cargo.toml#L155-L158>) 将 [`CountingAllocator<System>` 安装为宿主全局 allocator:3–7](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_app/src/lib.rs#L3-L7>)；[`CountingAllocator::realloc`:59–66](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/core/runtime/diagnostics/profiling/memory.rs#L59-L66>) 在底层返回非空后先减整块旧值，再加整块新值，[`Counters`:14–23](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/core/runtime/diagnostics/profiling/memory.rs#L14-L23>) 允许别的线程的完整分配/释放插在这两个原子操作之间。
实用读链：[`runtime_preview::main`:19–36](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_app/src/bin/runtime_preview.rs#L19-L36>) → [`EntryRunner::run_runtime_with_args` 启动采集:533–542](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_app/src/entry/entry_runner/runtime.rs#L533-L542>) → [`present_redraw_frame` 的宿主 profile scope:17–23](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_app/src/entry/runtime_entry_app/surface_present/redraw.rs#L17-L23>) → [`profile_frame!`:52–65](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/core/runtime/diagnostics/profiling/macros.rs#L52-L65>) → [`ProfileFrameScope::drop`:158–172](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/core/runtime/diagnostics/profiling/scope.rs#L158-L172>) → [`finish_frame`:296–300](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/core/runtime/diagnostics/profiling/scope.rs#L296-L300>) → [`record_frame`:69–81](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/core/runtime/diagnostics/profiling/memory.rs#L69-L81>) 读取 live/peak；事件循环退出后[导出采集:661–669](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_app/src/entry/entry_runner/runtime.rs#L661-L669>)。
UE 的普通非事务 [`FMemory::Realloc`:712–715](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Core/Private/HAL/UnrealMemory.cpp#L712-L715>) → [`FMemory_ReallocInline`:108–124](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Core/Public/HAL/FMemory.inl#L108-L124>) → `GMalloc->Realloc`，首次无 `GMalloc` 时 [`ReallocExternal`:533–541](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Core/Private/HAL/UnrealMemory.cpp#L533-L541>) 触发[从 `BaseAllocator` 初始化:348–360](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Core/Private/HAL/UnrealMemory.cpp#L348-L360>)。
以统计开启的 Win64 非 Shipping、选择 `-binnedmalloc2` 的[参数分支:174–205](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Windows/WindowsPlatformMemory.cpp#L174-L205>)、[`BaseAllocator` 实例化:257–259](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Windows/WindowsPlatformMemory.cpp#L257-L259>) 为例， [`FMallocBinned2::Realloc`:811–863](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Core/Private/HAL/MallocBinned2.cpp#L811-L863>) 原地 resize 分支对 requested large-pool counter 只做**一次** `new − old` 的原子更新。
## 2. 有界实际影响。

只影响启用该 allocator 包装器的宿主 Rust 堆诊断，不影响指针、实际分配结果、游戏结果，也不是 RSS、驱动内存或跨 DLL 统一记账。用 `X` 表示不变的其他 live 字节，假定已有峰不足以覆盖本次重叠。线程 A 原地 resize `1024 → 1536`，底层已成功，ZE 做完 `−1024` 时暂停；B 分配 `1024`、更新峰值、释放，再由 A `+1536`。记录器看到的本次 live 序列是 `X+1024 → X → X+1024 → X → X+1536`，最终 peak 只有 `X+1536`；B 存活时 A 的同一块并未消失，底层请求字节同时存在 `X+2560`，即使把 A 的逻辑更新视为尚未提交也至少 `X+2048`。这既可产生瞬时过低的 `memory.live_bytes`，也会遗漏 `memory.peak_bytes`（之后 stop/snapshot 不能补回漏过的历史；更晚出现的更大峰值可能掩盖它）。该交错无需定义 B 跟踪的实际墙钟频率；生产出现概率未测。
## 3. 根因与 UE 实际方法。

两个独立原子 read-modify-write 不组成一个事务，即使把顺序改成 SeqCst 也无法阻止 B 插入；当前峰值只来自每次 `allocated` 的 `fetch_add` 返回结果。[UE binned stats 宏:88–97](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Core/Public/HAL/MallocBinnedCommon.h#L88-L97>) 仅在该统计开启时生效；Binned2 原地分支在不改变 OS 分配尺寸时，以一个 `fetch_add(Δ)` 修改 requested 统计，[`DumpAllocatorStats`:1052–1069](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Core/Private/HAL/MallocBinned2.cpp#L1052-L1069>) 可读该值。UE 移动分支是另分配、复制、释放（有实际重叠），不在这条等价断言内；LLM/trace 是独立观测，本报告不声称它们或所有 UE allocator 的所有统计都有全局原子快照。
## 4. Rust 沿用 UE 思路，不换架构。

成功的 Rust realloc 对 `live` 只提交一次净差额，让每一笔请求在统计中连续存在。保留 `std::alloc::System`、既有 `GlobalAlloc` 封装、`Relaxed` 单计数语义和 recorder；不引入指针表、全局互斥、C++ allocator 移植或跨映像新会计域。
## 5. ABI / ECS 差异。

UE 的池统计使用对齐后的大小，Rust 的 `Layout::size()` / `new_size` 是调用者请求字节；不要做数值横向比较。维持 ZE 成功 realloc 计 `allocations` 一次，失败不修改任何计数；保留 `memory.*` 名称、单位、序列化、C ABI 和 ECS（修法均不涉及 ECS）。不新增读取多原子字段的快照一致性承诺。
## 6. ZE 改法草案（未实现）。

在现有 private `Counters` 增加 `reallocated(old,new)`：按增/减分支用一次 `fetch_add(new-old)` 或 `fetch_sub(old-new)`（避免 signed/unsigned 边界）；用该原子 RMW 得到的结果更新峰值，而不是事后重载 live，以免尚未采样的其他线程释放抹去峰值。`CountingAllocator::realloc` 只在原底层成功分支调用；同尺寸不降峰，增长/收缩/失败保持现有分配次数约定。顺序为独立问题 Draft → 用隔离计数器和测试侧可控执行点验证交错（不用 sleep，也不把无关测试分配计入）及串行 grow/shrink/same/failure 控制 → 再修复。最低先证明所选顺序下覆盖住 B 存活时 A 的请求字节；若做产物验证，再检查明确装了 wrapper 的宿主采集是否导出正确 row，不拿普通 libtest 的采样字段存在当作 allocator 实测。

## 测试迁移/比对

| UE 现有测试 / 检索 | 输入 → 期望 | ZE 已有落点 / 差距 |
|---|---|---|
| [`FPagedArraySetNumTest`（`System::Core::Containers::TPagedArray::SetNum`）:517–556](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Core/Tests/Containers/PagedArrayTest.cpp#L517-L556>)（邻近容器控制，非统计 oracle） | 空、`MaxPerPage−1`、满页、满页+1、缩回一页；断言元素数与页数，不检查 allocator 统计或并发。 | [`memory_profile_accounting_retains_peak_after_free_and_resize`:88–100](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/core/runtime/diagnostics/profiling/memory.rs#L88-L100>) 是 ZE 自有串行计数对照，不调用底层 realloc；[`memory_profile_frame_exports_heap_counters`:102–122](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/core/runtime/diagnostics/profiling/memory.rs#L102-L122>) 只断言帧中存在三个字段。 |
| **未找到同契约 UE 原测试；交错为源码推导，非迁移。** 在 `Runtime/Core/Tests`（含 `HAL`/`Memory`/`Templates`/`Containers`/`ProfilingDebugging`）查 `FMemory::Realloc`、`FMallocBinned*`、`AllocatedLargePoolMemory`、`GetAllocatorStats`、`MemoryTrace`、`LLM`；在 `Engine/Source` 的 `*Test*.cpp` 复查 Binned2/大块统计名。 | A 的旧块和新块之间，不得出现把同一块整个从 live 计数中删掉的窗口；B 的短时分配计入历史 peak。 | 现有 `profiling/memory.rs` 与 profiler `tests/` 没有对应并发净差额回归；保留下轮独立证伪。 |

查重：本地 PR 快照 `#135–145` 无 allocator 计账。合入基线的 [#130](https://github.com/He-Jiahui/ZirconEngine/pull/130) 和 R01 review/migration 核心是配置、任务、模块；`1367f2765` 的 `memory H1` 是虚拟几何 debug availability 测试，与 heap wrapper 无关。当前 wrapper 原接线提交 `de561ff33` 的相关既有测试已列上表。这不扩展到 renderer、editor、jobs 主线。

## UE 只读源指纹（SHA-256）

| 源文件 | SHA-256 |
|---|---|
| [Build.version](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Build/Build.version>) | `3dcd8b6654872d013f2b00aa5f0ef1e0db89d4e8627ce820554675a27f228ea6` |
| [Core.Build.cs:8–20](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Core/Core.Build.cs#L8-L20>) | `2bae34201d978e95d778674f5293b80683c8844ba8963184cf306c20ab06e558` |
| [UnrealMemory.cpp](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Core/Private/HAL/UnrealMemory.cpp>) | `d1147990895c08af4e83d65d00d942f74bb1c3ef4d730f0ffe55aa66bd33772c` |
| [FMemory.inl](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Core/Public/HAL/FMemory.inl>) | `592a750dd1584e7d6392581ee38827a8e3e56ade32fc40a37101cae6b6dd6302` |
| [WindowsPlatformMemory.cpp](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Windows/WindowsPlatformMemory.cpp>) | `c121f362923bb796bd71ec9a0fea0d80cc8939bc9aa67d8f89edcdcc320f1bbc` |
| [MallocBinned2.cpp](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Core/Private/HAL/MallocBinned2.cpp>) | `031eb45f2734989f67cfe72784c78619bd1495fe752eb6818538dad09a58ce87` |
| [MallocBinnedCommon.h](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Core/Public/HAL/MallocBinnedCommon.h>) | `77f6c42b3a6ea64cd1b114b98c09f4b53d8ae7278b41399c0affe62563f238d8` |
| [PagedArrayTest.cpp](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Core/Tests/Containers/PagedArrayTest.cpp>) | `7bbc4d9358a0bc2bb4d73618e246bdd517b40b5f75805df6d48afb09d6746ce0` |
