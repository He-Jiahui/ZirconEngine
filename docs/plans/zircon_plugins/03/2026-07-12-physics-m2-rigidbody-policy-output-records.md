# Plugins 03 · Physics M2-T2 刚体策略产出记录

> 日期：2026-07-12
> 状态：`plugins_03_m2_t2_mass_ccd_sleep_body_type_windows_jolt_27_of_27_runtime_3_of_3_editor_compile_gate_passed`
> 父计划：[`../03-physics.md`](../03-physics.md)

## 完成范围

- 共享 Scene、同步态与资产契约补齐 `PhysicsMassProperties`、`PhysicsCcdMode::{Disabled, LinearCast}`、`PhysicsSleepPolicy::{Allow, Never}`；旧 `can_sleep` 字段、反射名和 property path 已硬切删除，不保留生产兼容别名或双写。
- `PhysicsMassProperties` 支持显式质量（可带惯性张量）与按形状体积、密度自动求质量；primitive 与 compound 体积由 backend owner 统一解析，非法密度、质量和零/非有限体积返回 typed backend error。
- builtin 与 Jolt body 创建都消费解析后的质量；Jolt 创建路径设置 motion quality、sleep permission 与可表达的惯性 multiplier。
- `BodyCommand` 补齐 `SetCcdMode`、`SetSleepPolicy`；BodyType 运行期切换改为原地更新 object layer 与 motion type，保留线速度，不因策略切换替换 generation-checked handle。
- change detection 对 BodyType、CCD、SleepPolicy 发增量命令；质量属性等必须重建的 authored 变化走明确重建路径。world sync 不再预先吞掉或静默跳过 authored mass。
- 反射、项目 IO 与 Scene property 读写暴露 `RigidBody.mass_properties.mode`、`RigidBody.mass_properties.density`、`RigidBody.ccd_mode`、`RigidBody.sleep_policy`，并补齐非默认值 round-trip、反射和运行期 mutation 测试。
- Jolt `runtime.rs` 的命令应用职责拆入 `command_apply.rs`，避免原生后端 owner 继续膨胀。
- 已返还跨计划编译阻塞：[`../../zircon_editor/editor/08/fixed-2026-07-12-rigid-body-sleep-policy-consumer-cutover.md`](../../zircon_editor/editor/08/fixed-2026-07-12-rigid-body-sleep-policy-consumer-cutover.md)。

## TDD 与验证证据

- RED job `e98b11eb9c88444991fa5bec826da08b` 到达 Physics crate，并只因计划要求的 M2-T2 类型/字段尚不存在而失败。
- Windows managed Jolt check job `10ac7bf754ed448b8f9b2c3ba67cc1f4` 通过；首次 job `004d1c1057494240997ea65de5a8d244` 仅暴露构建机缺少 `LIBCLANG_PATH`，修正为 Visual Studio LLVM 后转绿。
- 最终 Windows feature-on Physics library job `1d33853ff25e449d83ce7c7603942eed`：27 passed / 0 failed，覆盖 auto mass、显式惯性 multiplier、零体积拒绝、CCD/SleepPolicy 原地切换及 kinematic → dynamic 速度保留。
- Runtime reflection managed job `ee73fcf057b24dc98ccae030f58ddc78`：1 passed / 0 failed；同一新构建宿主直接运行 property mutation 与 physics/animation project round-trip：2 passed / 0 failed。
- Editor compile-gate managed job `c349b8ccfed047a0b23b0c33f9993584` 成功生成新测试宿主；直接执行为 0 failed、3042 filtered，证明旧 `can_sleep` 六个 `E0609` 已解除。
- `audit_plugin_structure.py --json` 报告所有插件结构 violation count 为 0；tracked Rust `can_sleep` 扫描、scoped rustfmt、scoped `git diff --check`、新增生产 panic/allow 扫描均通过。
- 最大的本切片生产 owner 为 Jolt `runtime.rs` 484 行；property write 440、reflection 339、command buffer 313、world sync 245、mass resolver 218、`command_apply.rs` 90，未继续堆叠接近约 1000 行的混合职责文件。

## 明确能力边界

- 当前 JoltC ABI 只能把显式惯性张量表达为 analytic primitive inertia 的统一倍数；非均匀或旋转惯性张量返回 typed `Unsupported`，不会静默丢弃。
- 当前 JoltC 每 body 只暴露 sleeping permission，因此本切片权威策略为 `Allow/Never`；计划草案中的 per-body 线速度阈值、角速度阈值和入睡时间尚未虚构为已实现能力。
- 当前契约没有 center-of-mass authored 字段；本记录不宣称任意质心偏移已实现。
- M2-T3 `QueryMode` 与 sweep 多命中排序是下一活动切片。
