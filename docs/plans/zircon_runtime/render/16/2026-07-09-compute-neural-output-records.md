# 16-compute-neural 产出记录归档

> 来源：[`16-compute-neural.md`](../16-compute-neural.md) 的 `## 状态与产出记录`。

## 状态与产出记录

| 日期 | 里程碑/切片 | 状态 | 产出 | 验证与证据 | 后续 |
|------|-------------|------|------|------------|------|
| 2026-08-13 | CN-M1 shared GPU readback owner and Editor viewport GPU product | source_complete_dynamic_validation_pending | `zr_rhi_wgpu::GpuReadbackQueue` 成为唯一三槽 staging/map/callback owner，具备 256-byte layout、grow/shrink、non-blocking poll、slot backpressure、ticket/abort 与 panic-safe callback；timer/HZB/HGI/VG/particles 等普通 consumer 已迁移。Editor 同-device viewport 常规帧发布三代有界 GPU snapshot 和 backend-neutral resource identity，retained UI external-image provider 直接消费；fallback 仅走共享异步 ring，显式 capture 与产品 submit 分离。 | 当前树静态二审 C0/I0；生产私有 `map_async`/`wait_indefinitely` 扫描、默认产品 zero-CPU-pixel contract、generation/bounded registry/mailbox、mutex 外 framework call 与 scoped diff 均已核对。未运行 current-source Cargo/WGPU/RenderDoc；1080p/4K、多 viewport、resize/device-loss、latency/drop/copy/RSS/power 与 PNG 仍无本轮动态数值。 | coordinator 执行 readback focused tests、timer latency、Editor direct/fallback product、resize/device-loss、多 viewport WPR/Tracy、RenderDoc，并把当前源码 PNG 写入 `docs/tests/runtime/render`；证据通过后再 fixed/accepted。 |
| 2026-06-23 | Render index 当前状态总览拆分 | CN-M1 部分完成,CN-M2~M4 未启动 | 从 docs/plans/zircon_runtime/render/index.md 的第 9 节迁入本计划；本行保留 16 Compute/Neural 的当前事实，render 总索引不再维护计划级明细。 | 文档重组；本次未改生产代码，render/index.md 只保留状态路由说明。 | 仍未完成：NN operators、graph executor、NN postprocess、统一 compute framework；验收缺口：需要 compute descriptor/readback/dispatch helper、NN CPU reference tests、e2e inference |
| 2026-06-15 | CN-M1 compute framework | 部分完成: compute executor 分散存在,统一框架未落地 | HZB、SSAO、postprocess exposure、contact shadow、particle/other passes 已各自手写 compute dispatch;但无统一 descriptor、dispatch indirect、readback 和 resource validation 框架。 | 计划 04 VC-M3、计划 05 LS-M4、计划 07 PP-M3-S1b 状态表记录多个 compute executor 已接入;本文件 `现状与差距` 指出仍是 executor 内手写。 | 抽象 compute pipeline descriptor、bind layout contract、dispatch/readback helper 和 diagnostics。 |
| 2026-06-15 | CN-M2 NN operators / NN plugin skeleton and operator V1 | 未启动: 神经网络支持空白 | 无 NN graph、tensor resource、operator registry 或 model import。 | 本文件 `现状与差距` 明确神经网络完全空白。 | 建立 NN plugin crate、tensor buffer ABI、NN operators/basic ops 与 CPU reference tests。 |
| 2026-06-15 | CN-M3 graph executor and end-to-end inference | 未启动: 依赖 CN-M2 | 无图执行器、schedule、barrier 或 frame-synced inference。 | 当前无相关实现或状态表证据。 | 实施 graph compiler、resource planner、dispatch chain 和 readback/e2e tests。 |
| 2026-06-15 | CN-M4 NN postprocess integration | 未启动: 等待 CN-M1-M3 与计划 07 | 无 NN upscaler/denoiser/postprocess pass。 | 计划 07 后处理状态表显示 LUT/uber 后续仍未完成,NN postprocess 未进入实现。 | 在 postprocess graph 稳定后接 NN pass、history/input resource contract。 |

### 参考实现精读笔记

`dev/UnrealEngine/.../NNEHlslShaders/Internal/NNEHlslShadersOperator.h`:

- `EElementWiseUnaryOperatorType`(Abs/Acos/…/Relu/Sigmoid/Tanh/HardSigmoid/HardSwish/LeakyRelu/Selu/Softplus/Softsign/Sqrt 等 ~30 项)、`EElementWiseBinaryOperatorType`(Add/Div/Mod/Mul/Prelu/Pow/Sub)、`EElementWiseVariadicOperatorType`(Max/Min/Mean/Sum):UE 按"输入元数"而非语义分类,逐元素族共用一个 shader 框架。Zircon 对应:`nn_elementwise.wgsl` 单模板 + 函数体注入,同思路;V1 只取 Relu/Sigmoid/Tanh/Silu + 四则,远小于 UE 清单(裁剪依据:首场景为后处理 CNN)。
- 头内注释 `//Not, //And … need boolean tensors`、`//BitShift … need integer tensors`:UE 把非浮点 dtype 算子整体注释排除 —— 佐证本计划 V1 仅 f32/f16 的 dtype 收口是同款取舍。

`dev/UnrealEngine/.../NNEHlslShaders/NNEHlslShadersGemm.usf`:

- `#define WORK_TYPE float` + `READ(x)/WRITE(x)` 宏:精度与存取抽象一层,fp16 切换不改算法体。Zircon:WGSL 无预处理,由 `shader_templates.rs` 文本替换承担同职。
- ALGORITHM 0–3 朴素档(`[numthreads(8,8,1)/(16,16,1)/(32,32,1)/(256,1,1)]`)、4–6 共享内存档(`GROUP_SIZE 8/16/32`,`groupshared WORK_TYPE SharedMemoryA/B[GROUP_SIZE*GROUP_SIZE]`)、7+ 多载入档(`GROUP_SIZE_X 16/32` × `LOAD_PER_THREAD 16/8`):UE 按尺寸选档。Zircon 取舍:V1 只移植 ALGORITHM 5(TILE=16)单档 —— 多档选择是纯性能优化,等 RenderStats 计时数据再决定,接口(模板特化键)不锁死。
- 共享内存档核心:`NumGroupSteps = ceil(K / GROUP_SIZE)`;越界线程置 0 仍参与加载(`Temp = 0; if (… < M && … < K) Temp = READ(A[AIdx])`),`GroupMemoryBarrierWithGroupSync()` 两道(覆写前/使用前),最终 `if (DispatchThreadID.y >= M || x >= N) return;` 才出界返回,写回 `Alpha * Result + GetBetaTimesC(...)` —— `nn_gemm.wgsl` 逐条对应(见 GPU 布局节)。
- `StackShapeA_StackShapeB_StackStrideA_StackStrideB[MAX_NUM_STACK_DIMENSIONS]` + `GetMatrixStackOffsets(GroupID)`:batch/广播 GEMM 经 GroupID.z 索引堆叠矩阵。Zircon V1 不支持 batch GEMM(后处理 CNN 的 GEMM 在尾部全连接,batch=1),`NnOpParams` 预留 batch 字段,模板不实现。

`dev/UnrealEngine/.../NNERuntimeBasicCpu/Private/NNERuntimeBasicCpuModel.h`:

- `FModelCPU`:静态 `ModelMagicNumber`/`ModelVersionNumber` + `SerializationLoad(uint64& InOutOffset, TConstArrayView<uint8>)`/`SerializationSave`/`SerializationSize` 三段式游标序列化 —— `.znn` 的 magic/version 拒载与"格式即权威"取自此;Zircon 用定长 header + 表偏移代替游标递归(算子已扁平,无嵌套 Layer 树)。
- `FModelCPU::Layer: TSharedPtr<Private::ILayer>` 与 `FModelInstanceCPU::Instance: TSharedPtr<ILayerInstance>`:模型(权重,共享)与实例(中间缓冲,每实例)分离;`RunSync(TConstArrayView<FTensorBindingCPU>...)` + `SetInputTensorShapes`。Zircon 对应:`NnModelAsset`(共享)/ `run_cpu` 调用期临时 tensor 表(每调用),V1 不做实例池 —— CPU 档定位是回落与对拍,不追吞吐。
- `WeakThis: TWeakPtr<FModelCPU>` 保活手法:Rust `Arc` 语义天然覆盖,无对应物。

`dev/Graphics/.../universal/Runtime/MipGen/MipGenerator.cs`:

- `m_SupportCompute = SystemInfo.supportsComputeShaders` + compute/raster 双路径(`MipChainRasterBlurExecutePass` 用 `DrawProcedural` 兜底):Unity 为低端设备保留 raster 回落。Zircon 取舍:不做双路径 —— compute 能力缺失时 feature 经 capability gate 整体关闭(`backend_types.rs` 既有 `supports_async_compute`/`supports_neural_compute` 位),与风险节"能力检测"一致。
- `ComputePackedMipChainInfo`(注释 "We pack all MIP levels into the top MIP level to avoid the Pow2 MIP chain restriction"):mip 打包进单层回避尺寸限制。Zircon 不需要:计划 04 HZB 尺寸恒为 2 的幂(`next_pow2(view)/2`),无此约束。
- 每 mip 循环 `cmd.SetComputeTextureParam(data.cs, kernel, _Source, …, srcMipLevel)` + `cmd.DispatchCompute(cs, kernel, DivRoundUp(dstSize.x, 8), DivRoundUp(dstSize.y, 8), volumeDepth)`:最小 compute 封装形态 = (cs, kernel, 绑定表, DivRoundUp dispatch)四元组 —— `ComputePassDescriptor` 的 (shader, entry_point, bindings, dispatch) 四字段即其声明式等价,`PerPixel` 的 `div_ceil` 对应 `DivRoundUp`;区别:Unity 在 pass 回调里命令式逐 mip 重绑,Zircon 经 graph 声明每 mip 一个 pass(`HzbBuilder` 的 mip 链由计划 04 按 4 级一批展开),culling/生命周期可见性换少量 pass 开销。

## 风险与回退

- wgpu 无 fp16 storage 时算力受限:能力检测选 f32 路径,模型转换器可烘 f32;性能不达标先限低分辨率输入。
- 算子覆盖不足导致模型转换失败:转换器给出"不支持算子"清单诊断,V1 明确只支持声明的子集,不做静默近似。
- 推理与渲染同队列竞争帧预算:V1 同 encoder 串行 + 耗时统计;异步 compute 队列依赖 wgpu 多队列演进,接口不锁死。
