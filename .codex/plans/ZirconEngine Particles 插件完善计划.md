# ZirconEngine Particles 插件完善计划

## Summary
- 目标：把 `particles` 从当前的插件注册占位推进到完整可用的粒子插件，先交付 CPU sprite 粒子闭环，再在同一资产/组件模型上接 GPU 仿真。
- 架构默认：`zircon_plugins/particles` 拥有粒子资产、组件、仿真、编辑器 authoring；`zircon_runtime` 只保留中性 render DTO、scene extract、RHI/render graph 执行点，不反向依赖插件实现。
- 参考取向：CPU v1 采用 Unity/Fyrox 的模块化发射器体验；数据层按 Niagara/Unity VFX Graph 预留 SoA、capacity、CPU/GPU parity、indirect draw 和 readback 扩展点。

## Key Changes
- **里程碑 1：粒子插件运行时模型**
  - 拆分 `zircon_plugins/particles/runtime/src/lib.rs`，新增 asset/component/service/simulation/render/interop 子模块。
  - 新增公开类型：`ParticleSystemAsset`、`ParticleEmitterAsset`、`ParticleSystemComponent`、`ParticleSimulationBackend::{Cpu,Gpu}`、`ParticlesManager`、`ParticleEmitterHandle`。
  - `ParticlesManager` 替代当前只记录 count 的 `DefaultParticlesManager`，负责资产实例化、播放/暂停/停止、显式 tick、快照、editor preview rewind。
  - CPU 数据结构采用 SoA 粒子池：alive、age、lifetime、position、previous_position、velocity、size、color、rotation、angular_velocity、seed、emitter index、free list。

- **里程碑 2：CPU 粒子闭环**
  - 支持第一批模块：spawn rate、burst、lifetime、shape(point/sphere/box/cone)、initial velocity、gravity、drag、color over lifetime、size over lifetime、local/world space、max particles clamp。
  - 运行时每帧流程固定为：scene/component 收集 -> CPU spawn -> CPU update -> bounds/sort metadata -> render extract。
  - 编辑器 rewind 和测试使用固定 `1/60` 步长；正常 runtime 视觉粒子使用 frame dt，物理联动后再走可选 fixed substep。
  - 所有随机使用显式 seed，保证相同资产、相同 tick 序列下结果可复现。

- **里程碑 3：渲染提取与透明绘制**
  - 复用现有 `ParticleExtract` 和 transparent pass，补齐真实 sprite 输出、材质/纹理句柄、bounds、back-to-front 排序。
  - 修正粒子 shader/管线的 alpha 行为：片元保留 alpha，transparent blend 开启，深度读开启但不写深度。
  - CPU v1 只交付 billboard sprite；ribbon、mesh particle、sub-UV 动画进入后续扩展，不阻塞 GPU 基线。
  - 插件禁用时不产生粒子组件 UI、不注册粒子 render feature、不提交粒子 extract。

- **里程碑 4：编辑器 authoring**
  - 将当前 `particles.authoring` 入口升级为真实工具面：资产模板、发射器列表、模块栈、曲线编辑、component drawer、preview/play/pause/stop/rewind/warmup。
  - editor 只持有作者态和 preview 控制，不成为 runtime world 的数据所有者。
  - 新增导入/模板操作：创建 CPU sprite 粒子资产、给实体添加 `ParticleSystemComponent`、打开粒子 authoring surface。

- **里程碑 5：可选物理与动画桥**
  - 物理桥通过 capability 检测接入，不让 particles 硬依赖 physics 插件；首版提供 external force、simple collider query、collision event、bounce/dampen 参数。
  - 动画桥参考 Niagara AnimNotify：一次性 spawn event、timed emission、normalized progress、curve 参数绑定。
  - 无 physics/animation capability 时，对应模块在 runtime 降级为 no-op，并在 editor 中显示不可用诊断。

- **里程碑 6：GPU 仿真后端**
  - GPU 后端复用同一 `ParticleSystemAsset`，只切换 `ParticleSimulationBackend::Gpu`。
  - 新增 GPU layout 编译：从模块栈生成 WGSL include/entry 配置，生成与 CPU SoA 对齐的 attribute layout。
  - GPU pass 顺序固定为：spawn/update compute -> alive/count/compact -> optional sort -> transparent render。
  - 资源包含双缓冲 particle buffers、free/dead list、instance count buffer、indirect draw args、debug/readback buffer。
  - GPU 不可用、feature 被禁用或 WGSL 编译失败时，默认回落 CPU 并输出诊断。

## Test Plan
- **CPU runtime**
  - `cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_particles_runtime`
  - 覆盖 deterministic seed、spawn rate 累积、burst、lifetime death、free list reuse、max particles clamp、local/world space、pause/stop/rewind。
- **Render/runtime integration**
  - `cargo test -p zircon_runtime`
  - 覆盖 `ParticleExtract` 非空、透明 pass 注册/禁用、shader alpha、bounds/sort、禁用插件无粒子 pass。
- **Editor plugin**
  - `cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_particles_editor`
  - 覆盖 authoring view、drawer、template、menu operation、capability gate、physics/animation capability 缺失诊断。
- **GPU backend**
  - GPU 里程碑内增加 layout parity、WGSL compile、buffer capacity clamp、indirect args、CPU fallback、small-case CPU/GPU parity readback 测试。
- **里程碑验收**
  - 每个里程碑末尾运行对应 compile/test 阶段，修复失败后记录验收证据，并更新 `docs/` 中粒子模块文档和相关 `related_code`/`tests` 头信息。

## Assumptions
- 首个可交付版本只要求 CPU billboard sprite 粒子完整可用；ribbon、mesh particle、light particle、decal particle、volume particle 不进入 CPU v1。
- GPU 后端不另建一套资产系统；CPU/GPU 必须共享资产描述、模块枚举、capacity 规则和测试用例。
- 粒子插件继续作为 `zircon_plugins` 独立 workspace 成员推进，不修改 root workspace 形态，不引入兼容 shim 或旧路径 re-export。
- 计划文档默认可落到 `docs/superpowers/plans/2026-05-02-particles-plugin.md`，实现时再按里程碑执行。
