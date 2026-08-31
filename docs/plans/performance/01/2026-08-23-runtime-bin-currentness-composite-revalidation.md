---
title: Runtime Bin Currentness Composite Revalidation
date: 2026-08-23
scope:
  - zircon_runtime/src/bin
status: static_complete_dynamic_pending
evidence_level: E3-composite
source_fingerprint: d9d238dbc4621f315b387df373878b63a5d74e47496c7346bafa4ff8e1f6b67a
owners:
  - docs/plans/optimize/zircon_runtime/85-runtime-asset-import-source-discovery-importer-recipe-subasset-derived-data-artifact-cook-package-incremental-build-worker-determinism-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/91-runtime-material-shader-module-graph-permutation-compiler-reflection-layout-pipeline-pso-cache-prewarm-hot-reload-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_tooling/03-export-preset-build-cook-pack-platform-bundle-release-review.md
  - docs/plans/optimize/zircon_tooling/18-executable-target-entrypoint-cli-process-receipt-product-qualification-review.md
references:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/ShaderCompiler/ShaderCompiler.cpp
  - dev/UnrealEngine/Engine/Source/Developer/DerivedDataCache/Public/DerivedDataBuildScheduler.h
  - dev/UnrealEngine/Engine/Source/Developer/IoStoreUtilities/Private/IoStoreWriter.h
  - dev/godot/core/io/pck_packer.cpp
  - dev/bevy/crates/bevy_render/src/render_resource/pipeline_cache.rs
---

# Runtime Bin Currentness Composite Revalidation

## 1. 结论

`zircon_runtime/src/bin/**` 当前共有 **42/42 个 Rust 文件、9,419 行、360,135 bytes、97 个 test marker**，工作树为 clean。它们是 6 组离线构建、验证和生成工具，不是逐帧 Runtime 产品循环；但 export pack 和 shader prewarm 直接决定 MVP 的构建迭代、冷启动准备、CI wall time 与大型项目峰值 RSS。

本轮用“旧全量逐文件审查 + 当前源码 owner 审查 + 变更差量复读”完成组合验收，而不是重复制造第二套结论：

| 模块 | 当前规模 | 当前性证据 |
|---|---:|---|
| `zircon_export_pack` | 6 文件 / 869 行 / 4 tests | Runtime85 当前源码 E3 审查 |
| `zircon_export_validate` | 4 文件 / 694 行 / 12 tests | 本轮 4/4 逐文件复读；7 月后只有 `args.rs` 的共享路径等价判断变更 |
| `zircon_font_sdf_bake` | 4 文件 / 243 行 / 1 test | 7 月 22 日 40/40 全量审查后源码未变 |
| `zircon_host_reflection_docs*` | 4 文件 / 79 行 / 0 tests | 7 月 22 日全量审查后源码未变 |
| `zircon_shader_ide_env` | 3 文件 / 204 行 / 2 tests | 7 月 22 日全量审查后源码未变 |
| `zircon_shader_prewarm` | 21 文件 / 7,330 行 / 78 tests | Runtime91 当前源码 E3 审查；当前树相对其冻结点只有测试/API 适配和一处切片比较适配 |

当前目录按“相对路径 + NUL + 原始 bytes + NUL”的有序 SHA-256 为 `d9d238dbc4621f315b387df373878b63a5d74e47496c7346bafa4ff8e1f6b67a`。静态覆盖可以验收；Windows Cargo、真实大项目 cold/warm、WPR/ETW 和 current-source 产品执行证据仍为 0，因此性能完成状态必须保持 pending。

## 2. 旧结论纠偏

7 月报告对 shader prewarm 的“多次无界扫描、没有依赖 DAG、同一源码反复验证”描述已经过时。当前实现已经具备 bounded asset inventory、source content identity、include module SCC/DAG、batch 内单 source 一次 Naga validation，以及 resource/permutation registry。后续计划必须保留这些基础，不能用旧瓶颈为理由推倒重写。

仍未闭合的核心问题不是某个 iterator 或 clone，而是 owner 与调度系统没有统一：prewarm 仍是串行 worker，没有全局 CPU/内存 admission、优先级、取消/supersede、跨进程 worker receipt，也无法证明生成的 source/permutation 正好命中产品 Renderer 的完整 PSO identity。export 则仍从手写 manifest 读取 raw source，并把整包输入与结果驻留在内存，而不是消费 canonical build graph 产生的 qualified cooked artifact。

## 3. 当前瓶颈与归属

| 优先级 | 当前事实 | 复杂度/影响 | 唯一 owner |
|---|---|---|---|
| P0/P1 | prewarm inventory、module graph、compiler、PSO/runtime cache 各自拥有 generation 与发布边界 | 大项目 compile storm 仍由串行 worker 主导；无公平性、取消、优先级和 exact-hit 证明 | Runtime91 |
| P0/P1 | export pack 输入是 raw source，writer/delta 围绕整包 bytes 工作 | 峰值 RSS 随总资产 bytes 增长；warm/incremental 不能按 immutable action/chunk identity 跳过 | Runtime85 + Tooling03 |
| P1 | 6 个工具 target 的 identity、CLI schema、artifact receipt 和 qualification 分散 | 构建成功/退出码 0 不能证明同代产物、Ready 或可发布资格 | Tooling18 |
| P2 | `zircon_export_validate` 同时 materialize generated-contents JSON 与 report JSON 后写出 | 空间为 `O(contents + report)`；单次 control-plane 成本，远低于 pack/prewarm owner 问题 | Tooling03，不单列新 owner |

`zircon_export_validate/args.rs` 从手写 Windows lowercase 比较改为 `ProjectPaths::same_lexical_path`，是共享路径正确性收敛，不是新增性能风险。其 `run.rs` 没有目录全扫、每帧调用或二次闭包遍历证据；直接把 JSON 改成流式写入只会增加双输出一致性复杂度，不能修复 Build/Cook/Pack 的结构断链，本轮不修改生产代码。

## 4. Unreal 源码约束

Unreal 的 `ShaderCompiler.cpp` 不是在 CLI 内同步循环编译全部输入：它维护 local/distributed ShaderCompileWorker、worker 数量与存活时间、job priority range、outstanding job 计数、异步结果处理和独立时间统计。该证据支持 Zircon 将现有 SCC/source identity 接入共享、可限额的 compile service，而不是仅把串行循环换成无界线程。

`DerivedDataBuildScheduler.h` 把 build execution、memory 和 cache priority 作为调度合同；`IoStoreWriter.h` 把 chunk、compression、flush/finalize 与 writer 生命周期收口。它们共同约束 Zircon 的目标形态：asset build action 产生不可变、可寻址的 target artifact，pack writer 消费 chunk/receipt 并在 bounded buffer 内完成容器写入。不能继续让 prewarm、export pack 和 Editor wizard 各自重建 source/asset truth。

Godot PCK writer 与 Bevy pipeline cache只作为轻量旁证：前者分离 payload 写入与目录元数据，后者显式区分 queued/creating/ready 并使用异步 task pool。架构上限仍以 Unreal 的 worker/build scheduler/container owner 为准。

## 5. 结构优化顺序

### M0：指标与同代证据

保留当前 inventory、SCC 和 source identity；先为 scan、DAG、validate、compile、cache lookup、write、queue wait、worker busy、RSS 与 I/O 建立同一 invocation receipt。没有 current-source executable/tool artifact identity 时，不允许把旧二进制数据写入验收结论。

### M1：统一 Build/Compile Scheduler

将 shader prewarm 编译 action 接入 Runtime85 的 canonical build graph 与共享 scheduler。work key 至少绑定 source/module graph digest、compiler/backend/device profile、layout ABI、permutation domain 和 generation；具备 bounded CPU/RSS、优先级、取消、supersede、single-flight、last-known-good 与 worker terminal receipt。

### M2：PSO Exact-Hit 闭环

Renderer 全部 shader module/layout/pipeline 创建点硬切到 Runtime91 的统一 authority。prewarm manifest 必须由真实产品 material/pass/quality/device closure 生成；warm 启动逐项证明 artifact/cache hit，不接受“生成过 WGSL”代替 PSO 命中。

### M3：Cook/Pack 硬切

export roots 解析为 qualified asset closure；pack 只消费 target cooked artifact/chunk receipt。writer 使用 bounded read/compress/write window 和外部/增量索引，delta 以 chunk identity 工作，不再让 raw source、整 target pack 与整 delta pack 同时驻留。

## 6. 动态验收矩阵

1. Shader：sources `1/100/10k`、includes `0/10/100`、variants `1/1k/100k`、WGSL `4 KiB/1 MiB`、cold/warm/supersede/cancel；记录 scan entries、DAG nodes/edges/SCC、validation count、compile count、queue p95/max、worker utilization、cache hit、exact PSO hit、peak RSS 与 wall time。
2. Pack：asset `1/100/10k`、payload `1 MiB/1 GiB`、dedup `0/50/99%`、cold/warm/resume/delta；记录 open/read bytes、chunk reuse、compression throughput、write amplification、peak RSS、temporary disk 与 output digest。
3. `zircon_export_validate`：small/large profile closure 下记录 output bytes、encode/write wall time 与 peak RSS，只作为 control-plane 基线，不抢占 MVP runtime/editor 产品热点优先级。
4. WPR/ETW 用于 CPU sampling、file I/O、context switch、thread/worker lifetime、power 和 RSS；RenderDoc 只用于 current-source Renderer 的 draw/pipeline/cache marker 与 GPU frame，不用于证明这些 CPU CLI 工具性能。
5. 必须先得到受管 Windows Cargo 构建出的 current-source artifact receipt。当前会话无可执行 validator，故本轮 Rust test、WPR 与 RenderDoc 均为 `0`，不得写入 `review.md`。

## 7. 本轮验证

- 源码 inventory：42/42 文件、9,419 行、360,135 bytes、97 tests。
- `git status --short -- zircon_runtime/src/bin`：0 项。
- 7 月报告后变更：15 文件；14 个 prewarm 文件由 Runtime91 当前审查覆盖，`zircon_export_validate/args.rs` 本轮复读。
- 当前 source fingerprint：`d9d238dbc4621f315b387df373878b63a5d74e47496c7346bafa4ff8e1f6b67a`。
- 生产代码改动：0。Rust/Cargo/WPR/RenderDoc：0。

