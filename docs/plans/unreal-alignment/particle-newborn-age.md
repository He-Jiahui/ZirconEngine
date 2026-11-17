# P01 / Draft 问题定义：CPU 粒子把帧内新生粒子按整帧推进，定时 burst 可在首次可见前被误杀

> 取证说明：UE 的本机路径与文件 SHA256 是用户提供参考树的依据；本轮 D: 审计快照/工作记录仅本地保存，不随本 PR 发布，也不冒充 GitHub 可读证据。UE/ZE 测试均未在本轮执行。

状态：**仅调查、一个独立问题**。粒子/VFX 领域只提出这一条；保持「清晰六点 Draft → 契约单测 → 可行修复」，本轮不实施后两步。不是“ZE 没有 Niagara 高级能力”的功能缺项报告。

- ZE 只读源码 HEAD `cb495bb1c952a24c86f4513b8099bc4942c3a87d`；lead 更正的远端基线 `e89349ca51b4e9d024c194cd3bdf645bd25b6c37` 与它在引擎 crates 的本地只读 diff 为空，故 ZE 证据一律固定为 **e893 blob**。
- UE 只读版本为 [Build.version:1–10][ue-version] 的 `6.0.0 / UE5`；不把嵌套路径向上继承的 ZE Git HEAD 冒充 UE 提交号。UE 仅短述、精确行号和 SHA256。
- 已读规范 [ZE AGENTS][ze-agents]、[任务树 AGENTS][task-agents] 及相关工程/dev 规则。外来 skill/catalog 脏文件未触碰。**未改代码/测试，未 Cargo/UBT，未运行 UE/GPU/产品，未 Git 写或远端操作。**以下数值是对已读源码的可核对推导，不是执行结果。

## 1. 两边差异、精确符号及可达性

### ZE：已有定时 burst 的发生时间被识别，却在更新阶段丢失

真实 owner 不是 runtime 猜测的粒子目录，而是独立 plugins workspace 的 [particles/runtime/Cargo.toml:1–14][ze-particle-cargo]，包名 `zircon_plugin_particles_runtime`。

- [asset.rs:118–130、177–198][ze-asset] 的 `ParticleBurst::{time_seconds,count}` 与 `ParticleEmitterAsset` 已明确支持定时发射、寿命与速度；[editor/source/burst.rs:19–25][ze-burst-source] 明确说明 burst 用系统模拟秒，不是归一化生命曲线。不是新增子帧作者功能。
- [editor/source/lower.rs:29–34、73–82][ze-lower] 的 `ParticleSystemSource::to_runtime_asset` 将原作者 `time_seconds` 原样 lower 为 `ParticleBurst`。
- [service.rs:193–221、245–251][ze-service] 的公开 `ParticlesManager::{instantiate,tick}` → [simulation/cpu.rs:85–101][ze-cpu] 的 `ParticleSystemInstance::tick`。后者先 `spawn_due_particles(previous_age,current_age,...)`，再 `update_particles(scaled_dt)`。
- [cpu.rs:204–237][ze-cpu-spawn] 的 `spawn_due_particles` 检查各 burst 时间是否落入本 tick，并累计发射数量；但把全部时间压成一个 `spawn_count`，后续 `initial_particle` 已不知道何时出生。
- [pool.rs:64–83、94–108][ze-pool] 的 `CpuParticlePool::spawn/write` 将 newborn age 置 0；[cpu.rs:279–310][ze-cpu-update] 随即对全部 alive 粒子统一 `age += dt`，`age >= lifetime` 立刻 kill。因此 newborn 会被计入出生以前的时间。
- [service.rs:323–356][ze-snapshot] 的 `snapshot` → [cpu.rs:313–346][ze-cpu-extract] 的 `append_sprites` 只输出 alive。首次 tick 内误杀的粒子不会进入首次可见快照；[service/preview.rs:97–134][ze-preview] 的 `step_preview` 也实调相同 instance tick，不是另一套模拟。

### 最小非边界 witness（静态推导，未执行）

新 manager，Cpu、playing=true、time_scale=1；单个 Point emitter，`spawn_rate=0`、`burst=(0.75秒,1)`、`lifetime=0.5秒`、`max_particles=8`、恒定速度 `(1,0,0)`、gravity/drag=0、identity transform。一次 `tick(1.0)` 后读取真实 snapshot。

| 阶段 | 当前源码推导 | 基础时间契约 |
|---|---|---|
| `[0,1]` 中识别 burst | 0.75≤1，spawn_count=1 | 正确，本帧应出生一次 |
| 初始化 | age=0，position=0 | 正确 |
| 更新 | 给 newborn 加整帧 1.0，1.0≥0.5，kill | newborn 只存在 `1.0−0.75=0.25` 秒，应仍 alive |
| snapshot | `live_particles=0`，sprites 空 | 应 1 个 sprite，position.x=0.25；内部 age=0.25 |

这里 0.75/0.5/0.25/1.0 均可由二进制浮点精确表达，没有随机形状、容量压力、外力、碰撞或积分精度争议。初始速度与寿命是 constant，结论不依赖 seed。

### UE：先区分已有粒子与新生粒子，必要时只推进出生后的余量

- 构建 owner：[Niagara.Build.cs:12–47][ue-niagara-build] 依赖 Engine/Renderer、NiagaraCore/Shader/VertexFactories、VectorVM、RenderCore/RHI；生成出生插值代码在 [NiagaraEditor.Build.cs:10–63、85–102][ue-niagara-editor-build] 对应 Editor 模块。
- 实际运行链：[NiagaraSystemInstance.cpp:2681、2736–2743][ue-system] 的 `FNiagaraSystemInstance::Tick_Concurrent` → `Emitter.Tick(CachedDeltaSeconds)` → [NiagaraEmitterInstanceImpl.cpp:1255、1346–1362][ue-emitter] 的 `FNiagaraEmitterInstanceImpl::Tick`。
- [NiagaraEmitterInstanceImpl.cpp:1615–1639][ue-emitter-update] 仅对 `OrigNumParticles` 调用 UpdateExecContext；然后 [同文件:1688–1763][ue-emitter-spawn] 才执行 SpawnExecContext。新粒子不被不加区分地塞进原有粒子的整帧 Update。
- [NiagaraTypes.h:331–359][ue-spawn-info] 的 `FNiagaraSpawnInfo` 保留 `Count`、首个粒子的 `InterpStartDt`、粒子之间的 `IntervalDt`；[NiagaraEmitterInstanceImpl.cpp:1753–1761][ue-emitter-spawn] 将它们绑定到实际 spawn 执行参数。
- 可选插值链：[NiagaraHlslTranslator.cpp:3370–3419、3464–3465][ue-translator] 的 `DefineInterpolatedParametersFunction` 为每个 newborn 计算出生时刻及 `frame_dt−spawn_time`，并将后者写给更新阶段的 `Engine.DeltaTime`。所以不是只存 metadata 没有消费者。

限定：UE 的 NoInterpolation / RunUpdateScript / Interpolation 是不同路径。本稿不把所有 Niagara 资产都称为精确子帧模拟，也不声称已执行 UE 模块脚本证明 age 数值；借鉴的是**已有/新生 cohort 分开，不能把出生前 dt 算进 newborn**。本地传统 Engine/Private/Particles 只查到 FX/perf 辅助源码，未找到 Cascade emitter 实现，本稿不用虚构 Cascade 行号替代 Niagara。

## 2. 风险边界：非边界、恰好边界与 continuous 分开

| 情形（均为源码推导，未执行） | 当前行为 | 本问题对应的边界 |
|---|---|---|
| 非边界 burst=0.75，tick=1，life=0.5 | 首帧即死，0 个 | 只活过 0.25 秒，不应先死 |
| 右边界 burst=1，tick=1，life=0.5 | 已纳入该 tick，整帧 aging 后死 | 现有 due 条件包含 current_age；如本帧发射则 age 应为 0，下帧不得再次发射；不是要求改变包含边界约定 |
| 起点 burst=0，tick=1，life=0.5 | age=1，死亡 | **正确对照**；不能简单交换 spawn/update，把所有 t=0 burst 冻成 age=0 |
| 同一个 0.75 witness 改成八个 tick=0.125 | t=0.75 首次出生即 age=0.125；到 t=1 age=0.375、仍活，x=0.375 | 与一次 tick=1 的空快照不一致；正确应均为 age/x=0.25。此处无重力/drag，差异不是一般 Euler 分步误差 |
| 连续 rate=4/s，首个 tick=0.25，life=0.125 | accumulator 达 1 时出生，随后 age=0.25 即死 | 对按 rate 阈值到达时出生的既有模型，首个出生在帧右端，age 应 0。不是把 continuous 当作 t=0 burst |

风险仅限 CPU path 的出生/首帧年龄、运动/旋转/生命曲线或寿命终止；短寿命和大 dt 更易出现可见缺粒子。时间步上限、产品帧频、受影响内容数量本轮未测。无证据声称所有效果不可用、存在内存越界、GPU 全部相同或必须实现 Niagara 任意图/事件/高级求解器。

## 3. 根因及 UE 实际解决

根因在于**发射计划只保留数量，不保留本 tick 的出生时间，更新又遍历所有 alive 槽**。burst 判断 `previous_age/current_age` 已知道出生何时发生，但这个信息在 `spawn_count` 聚合处丢失；pool 的 age=0 之后被整帧 dt 覆盖语义。

UE 不是靠无限小 tick 掩盖：运行时明确先更新 `OrigNumParticles`，然后执行 spawn；开启插值时还用 `InterpStartDt/IntervalDt` 得出 newborn 剩余时间、改写实际 Update DeltaTime。ZE 不需要 UE 编译 VM，也能保留已有 burst 时刻并分别计算余量。

## 4. Rust 直接借鉴思路，不新架构

在现有 `ParticleEmitterInstance` / `CpuParticlePool` 内分开 old/newborn：old 仍用完整 `scaled_dt`；newborn 用 `current_age−birth_time`。复用现有单粒子的年龄、存活、速度、位置、旋转与曲线更新逻辑，不只改 `age` 而让位置/颜色仍使用全帧 dt。

burst 的 birth_time 直接来自现有 `ParticleBurst`；continuous 使用进入本帧前的 accumulator、rate 和第几个越过整数阈值得出出生时间。保持一次发射一次、按原 seed 顺序采样、既有 Local/World transform、time_scale 只乘一次和 snapshot 缓存失效规则。无需新 ECS 调度器、事件总线、通用 Niagara graph 或 ABI 对象。

continuous 的精确出生 phase 及其旧 lifetime fixture 的取舍仍须先冻结语义；P-T5 是条件化契约草案，不是 UE 原测试或可机械改断言的依据。本问题放行独立 Draft 依赖的是非边界 burst witness，不依赖 continuous 假设。

**不能只交换两行调用**：那会让起点 burst 不推进；也不能人为拉长 lifetime 或 clamp 所有 dt 来掩盖本问题。需用第 2 点的起点/右端/非边界控制区分正确修复。

## 5. wgpu / ABI / ECS 差异

- 当前 CPU pool 是 Rust SoA；改出生余量不要求改变 `RenderParticleSpriteSnapshot` 或 runtime/plugin ABI。保留 `ParticleEmitterHandle`、资源句柄及稳定 sprite key。
- GPU 是另一条实际实现：[gpu/shaders.rs:217–244、246–271、274–299][ze-gpu-shader] 的 `particle_spawn_update` 按输入 alive 分支：已有粒子 update，新生粒子 spawn 后 age=0，**不会继续落入 update**。因此不能把 CPU 误杀直接宣称成所有 GPU 同样误杀。
- [gpu/planner.rs:194–237、270–301][ze-gpu-planner] 目前传 spawn_count，不传逐粒子 birth time；它的帧量化语义与 CPU 精确 burst 余量需要单独审查。本稿不以“未做 GPU 子帧插值”另报第二 bug，也不要求把 Niagara cbuffer/VM layout 搬进 wgpu storage。
- [service.rs:200–216][ze-service] 明确存在 GPU 请求的 CPU fallback；仅凭资产 backend=Gpu 不能证明执行过 GPU。[VFX plugin.rs:159–173、207–235、253–266][ze-vfx] 确实复用粒子管理器与 GPU prepare owner，但 VFX 产品输出是否走 fallback、本问题对其可见帧的影响，本轮未知，**不据此再开 VFX 问题**。
- ECS/entity 在此用于 component/sprite 归属；本轮只核到公开 manager 与 preview 的可达链，没有宣称普通 World tick 自动调度全部可选插件。无需更改 mesh affine、frustum、TBN/Mikk、物理世界姿态写回。

## 6. ZE 具体修改草案（未实施）

| 现有文件 / 符号 | 窄修改方向 |
|---|---|
| [simulation/cpu.rs::ParticleSystemInstance::tick][ze-cpu] | 明确 old cohort 和本帧新生 cohort 的更新边界，保留现有 dt 验证/playing/time_scale |
| [cpu.rs::ParticleEmitterInstance::spawn_due_particles][ze-cpu-spawn] | 不再将不同发生时刻的 burst/continuous 全压成无时间 spawn_count；对本帧每个出生批次保留 residual dt，尊重现有 next_burst_index |
| [cpu.rs::update_particles][ze-cpu-update] | 抽出同一单槽更新函数以复用全帧/余量逻辑；不能给 newborn 重复做两次 force/drag/曲线 |
| [pool.rs::spawn/write][ze-pool] | 如需按 newborn 槽更新，可让现有 spawn 返回被使用的 slot index；不引入第二套 pool，不改变外部快照 ABI |
| [tests/cpu_simulation.rs][ze-cpu-tests]、[tests/extract.rs][ze-extract-tests] | 先加入下表时间见证与快照断言，保留已有 seed、暂停/停止、提取 metadata 控制 |
| [editor/tests/source_burst.rs][ze-editor-burst-tests] | 用现有 source→lower→manager 的真实链加短寿命 witness，不只断言作者时间字段被保存 |

容量与复用注意：调整更新顺序可能改变“本帧死亡是否立即让出容量”的邻接行为。后续修复必须在现有 max_particles/free-list 测试下显式检查，不把这项策略扩展当成 P01 已证明的独立缺陷。最小 witness 无容量压力，可先独立锁住出生年龄问题。

既有 `cpu_particles_apply_lifetime_death_and_reuse_allocated_slots` 在 [cpu_simulation.rs:39–61][ze-cpu-tests] 用 rate=20、life=0.06，并期望 tick0.05 后再 tick0.02 已死，实际依赖了“首个 continuous newborn 在第一帧被预老化0.05”的旧假设。未来不能只改断言压绿：应先用独立 witness 证明 RED；若保留该测试的死亡/槽复用目标，可改为 rate=0、显式 t=0/t=0.12 两次 burst，保留 1→0→1 与 allocated_slots=1 的断言。该 fixture 修订、语义取舍和所有执行均尚未进行。

## UE 既有测试迁移表与拟测契约

**本轮没有找到 newborn/burst 出生年龄的可直接迁移 UE 既有测试。** 已搜索 Niagara/Source 下测试命名文件、Niagara runtime 的 automation/TEST_CASE 注册，以及整个 Niagara 下 `*Spawn*Test*` 路径；读了 runtime ISPC vector field 测试、Editor inheritance 与 stack 测试。没有执行或穷举所有 Niagara 二进制功能测试资产。因此下面 P-T* 明确是**源码推导契约，非 UE 单测迁移**。

| UE 既有测试名 / 精确路径行 | 输入 | 期望 | 与本问题的迁移结论 / ZE 现成落点 |
|---|---|---|---|
| `FIspcTestNiagaraDataInterfaceVectorField` / `Ispc.Graphics.NiagaraDataInterfaceVectorField`，[IspcTestNiagaraDataInterfaceVectorField.cpp:14–45、155–161][ue-vector-test] | 同一静态 vector field，ISPC 与 C++ sample | XYZ 输出相等 | 非出生时序，**不迁移成 P01 覆盖** |
| `FNiagaraEditorDiffSameEmittersTest`，[NiagaraEmitterInheritanceTests.cpp:188–206][ue-inheritance-test] | 装载 emitter 后 duplicate，diff 原件/副本 | diff 有效且四个脚本 stack diff 为空 | 非模拟 age；不移植其内容依赖 |
| `FNiagaraEditorStackViewModelAddParameterModulesTest`，[NiagaraStackViewModelTests.cpp:31–81][ue-stack-test] | 增加参数模块的编辑器样例 | 原样例断言模块/输入存在 | **代码在注释块中，不是已启用覆盖**；不拿来证明运行契约 |
| 本次未找到对应的出生余量既有 UE 测试 | — | — | 转为以下源码契约测试草案，不伪称迁移 |

| 拟测契约（未新增、未运行） | 输入 / 期望 | ZE 已有测试落点 |
|---|---|---|
| P-T1 非边界 burst | 本文 0.75/0.5/1 witness：snapshot 1 个，x=.25；内部 age=.25（可选白盒断言） | [runtime/tests/cpu_simulation.rs][ze-cpu-tests]、[runtime/tests/extract.rs:41–89][ze-extract-tests] |
| P-T2 边界不重发 | burst=1，tick1 后 age/x=0、live=1；再 tick.125 仍只一个，age/x=.125 | 同上；与现有 [editor source_burst 实际发生测试:44–75][ze-editor-burst-tests] 配对 |
| P-T3 起点/寿命控制 | burst=0，life=.5，tick1 应死亡；t=0、life=2、tick.25、angular_velocity=2 时保持现有 rotation=1.0 控制 | [extract.rs:41–89][ze-extract-tests]；不把“交换顺序导致所有 newborn 不更新”判成通过 |
| P-T4 分步不改变无力运动 | 同资产一次1 vs 八次.125，t=1均 live=1、x=.25；零gravity/drag避免一般数值积分差异 | [cpu_simulation.rs:10–36][ze-cpu-tests] 既有 deterministic harness，只换成独立时间契约而非比较相同 tick 表 |
| P-T5 continuous 独立控制 | rate4、tick.25、life.125，首个粒子应在右端 birth，live=1、age/x=0；后续阈值/accumulator 不丢发 | [cpu_simulation.rs][ze-cpu-tests]、[support.rs:16–30][ze-test-support] |
| P-T6 作者端可达 | 从 `ParticleSystemSource` 输入非边界短寿命 burst，lower 后 manager 的 snapshot 仍符合 P-T1；原始源顺序/稳定 key/逆操作不变 | [editor/tests/source_burst.rs:9–75][ze-editor-burst-tests]，不用只看 TOML 往返 |

以上期望中的源码依据为 [UE old/newborn 执行分离][ue-emitter-update]、[spawn time 数据契约][ue-spawn-info]、[residual dt 的实际生成/消费][ue-translator] 和 ZE 已有系统秒 burst 语义；**没有运行 UE 或 ZE 证明这些测试通过/失败**。

## 模块 / owner / 覆盖 / 未知清单

| 子域 | 真实模块 / owner | 已检查覆盖 | 未知 / 不报项 |
|---|---|---|---|
| CPU 粒子、基础组件 | [particles runtime manifest][ze-particle-cargo]；[ParticlesManager][ze-service]、[simulation/cpu.rs][ze-cpu] / pool | 发射、寿命、取快照、公开 tick、preview；现有 CPU deterministic/death/rewind 测试只读 | 本机编译/执行、自动 World 插件调度、产品显示未验证 |
| 粒子 authoring | [source/burst.rs][ze-burst-source]、[source/lower.rs][ze-lower]、[tests/source_burst.rs][ze-editor-burst-tests] | 实际系统秒语义、source lower、已有 manager 调用；既有测试覆盖发生数量但未覆盖短寿命 newborn age | UI 输入、保存、undo 的产品集成未运行；不新增第二作者问题 |
| GPU 基础模拟 | [GPU planner][ze-gpu-planner]、[generated WGSL][ze-gpu-shader]；[GPU tests:494–555][ze-gpu-tests] | 当前 spawn 分支 age0，不走 old update；现有 parity 只比较长寿命/t=0/tiny dt 的 counts/indirect | GPU 精确 birth remainder 与 age/position parity 未测；不将 missing advanced interpolation 当 bug |
| VFX graph | [vfx_graph/runtime/Cargo.toml][ze-vfx-cargo] / [VfxRuntime][ze-vfx]，依赖 particles 与 shader_graph；[tests:55–152][ze-vfx-tests] | 已核真实 lowering/configure/tick/prepare 链，测试源码包含真实产品 GPU buffer probe | VFX 测试未执行；该 test 默认长寿命且未核出生 age，不能为 P01 背书；不另列 VFX issue |
| UE/build owner | [Niagara.Build.cs][ue-niagara-build]、[NiagaraEditor.Build.cs][ue-niagara-editor-build]；ZE [plugins workspace:66–68、120–121][ze-workspace] | 区分 runtime execution 与 Editor HLSL 编译；粒子/VFX 不在根 workspace 直接成员中猜测 | 人工 CODEOWNER 未确认；不虚构人名或 full parity 结论 |

范围外/未知保留：ribbons、mesh particles、Niagara 任意脚本、事件图、GPU 子帧精确模拟、复杂 collision、跨平台 determinism、所有特效产品效果。本轮不以这些缺项硬凑问题。

去重：已读 [lead PR snapshot][pr-snapshot]（SHA256 `B374EE7808D92536D2680A200DE47479F5A458015F568562465B209C0AB0EF30`，其范围 #131–145）。open #135/#144 是 mesh/editor affine，#138 是裁剪，#140/#141/#143 是 TBN/Mikk/fixture，#145 是物理姿态写回，#137/#139/#142 是模型/插件测试问题，#136 是流程；P01 只改粒子出生时间→年龄/首帧积分，和这些输入、根因、owner、测试落点不同。粒子 CPU/GPU 差异与 VFX 共享后端只作本问题边界说明，不拆成重复 issue。未声称遍历所有历史 PR。

## UE 文件 SHA256（未复制大段源码）

| 文件 | SHA256 |
|---|---|
| [Build.version][ue-version] | `3DCD8B6654872D013F2B00AA5F0EF1E0DB89D4E8627CE820554675A27F228EA6` |
| [Niagara.Build.cs][ue-niagara-build] | `2DBFF84D077150B35FD00C82E4BB43963F3F68FEF128D63951465DDBA471D882` |
| [NiagaraEditor.Build.cs][ue-niagara-editor-build] | `E4690E5449E518CF1470B7096586150FC1E5EE398C7695AC45B26CE0D28CAD51` |
| [NiagaraTypes.h][ue-spawn-info] | `357B897A9AF8B3B54B9B9F9A0F4881556CBD3B3EF9ECCBA12D85C7BDF049368B` |
| [NiagaraSystemInstance.cpp][ue-system] | `6EFE586494DE5E562D314AB2CEB0EBBCFEF84FF78A6BB41FD9C5AF7BF9B51226` |
| [NiagaraEmitterInstanceImpl.cpp][ue-emitter] | `415351C1291023129B83BE17C7FC5AA75ACC6D778B58B29CAF271EF596F957CF` |
| [NiagaraHlslTranslator.cpp][ue-translator] | `2A16B804AA8212540553D223AAE56FDC526EB869E99F717A76B30B67DC62AE97` |
| [IspcTestNiagaraDataInterfaceVectorField.cpp][ue-vector-test] | `3B95154B4346C86D419BECB0A67EF8418B3DEEDAF43A42F794408237EDAB5D85` |
| [NiagaraEmitterInheritanceTests.cpp][ue-inheritance-test] | `D2E73A265B0049E3A279BDC1DF4701D1FA972C67BB0D64FADF5F58DE41BFE6F9` |
| [NiagaraStackViewModelTests.cpp][ue-stack-test] | `BC80B534B219D3A50C1B7B30C19828955F525299AF9EFBC9AE1BB6D3F90218F8` |

[ze-agents]: <E:/Git/ZirconEngine/AGENTS.md>
[task-agents]: <D:/CodexData/worktrees/ze-ue-audit-20261117/AGENTS.md>
[pr-snapshot]: <D:/CodexData/worktrees/ze-ue-audit-20261117/reports/ue-module-audit-20261117/github-pr-snapshot.json>
[ze-particle-cargo]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/particles/runtime/Cargo.toml#L1-L14>
[ze-asset]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/particles/runtime/src/asset.rs#L118-L198>
[ze-burst-source]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/particles/editor/src/source/burst.rs#L19-L25>
[ze-lower]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/particles/editor/src/source/lower.rs#L29-L82>
[ze-service]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/particles/runtime/src/service.rs#L193-L251>
[ze-cpu]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/particles/runtime/src/simulation/cpu.rs#L85-L101>
[ze-cpu-spawn]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/particles/runtime/src/simulation/cpu.rs#L204-L237>
[ze-cpu-update]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/particles/runtime/src/simulation/cpu.rs#L279-L310>
[ze-pool]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/particles/runtime/src/simulation/pool.rs#L64-L108>
[ze-snapshot]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/particles/runtime/src/service.rs#L323-L356>
[ze-cpu-extract]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/particles/runtime/src/simulation/cpu.rs#L313-L346>
[ze-preview]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/particles/runtime/src/service/preview.rs#L97-L134>
[ze-gpu-shader]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/particles/runtime/src/render/gpu/shaders.rs#L217-L299>
[ze-gpu-planner]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/particles/runtime/src/render/gpu/planner.rs#L194-L301>
[ze-cpu-tests]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/particles/runtime/src/tests/cpu_simulation.rs#L10-L82>
[ze-extract-tests]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/particles/runtime/src/tests/extract.rs#L41-L89>
[ze-editor-burst-tests]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/particles/editor/src/tests/source_burst.rs#L9-L75>
[ze-test-support]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/particles/runtime/src/tests/support.rs#L16-L30>
[ze-gpu-tests]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/particles/runtime/src/tests/gpu.rs#L494-L555>
[ze-vfx-cargo]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/rendering/features/vfx_graph/runtime/Cargo.toml#L1-L12>
[ze-vfx]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/rendering/features/vfx_graph/runtime/src/plugin.rs#L159-L266>
[ze-vfx-tests]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/rendering/features/vfx_graph/runtime/src/tests/lib.rs#L55-L152>
[ze-workspace]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/Cargo.toml#L66-L121>
[ue-version]: <E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Build/Build.version#L1-L10>
[ue-niagara-build]: <E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Plugins/FX/Niagara/Source/Niagara/Niagara.Build.cs#L12-L47>
[ue-niagara-editor-build]: <E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Plugins/FX/Niagara/Source/NiagaraEditor/NiagaraEditor.Build.cs#L10-L102>
[ue-system]: <E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Plugins/FX/Niagara/Source/Niagara/Private/NiagaraSystemInstance.cpp#L2681-L2743>
[ue-emitter]: <E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Plugins/FX/Niagara/Source/Niagara/Private/NiagaraEmitterInstanceImpl.cpp#L1255-L1362>
[ue-emitter-update]: <E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Plugins/FX/Niagara/Source/Niagara/Private/NiagaraEmitterInstanceImpl.cpp#L1615-L1639>
[ue-emitter-spawn]: <E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Plugins/FX/Niagara/Source/Niagara/Private/NiagaraEmitterInstanceImpl.cpp#L1688-L1763>
[ue-spawn-info]: <E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Plugins/FX/Niagara/Source/Niagara/Public/NiagaraTypes.h#L331-L359>
[ue-translator]: <E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Plugins/FX/Niagara/Source/NiagaraEditor/Private/NiagaraHlslTranslator.cpp#L3370-L3465>
[ue-vector-test]: <E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Plugins/FX/Niagara/Source/Niagara/Private/IspcTestNiagaraDataInterfaceVectorField.cpp#L14-L161>
[ue-inheritance-test]: <E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Plugins/FX/Niagara/Source/NiagaraEditor/Private/Tests/NiagaraEmitterInheritanceTests.cpp#L188-L206>
[ue-stack-test]: <E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Plugins/FX/Niagara/Source/NiagaraEditor/Private/Tests/NiagaraStackViewModelTests.cpp#L31-L81>
