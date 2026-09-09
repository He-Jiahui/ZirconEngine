---
related_code:
  - zircon_runtime/src/core/framework/animation/compiler/sequence/compile.rs
  - zircon_runtime/src/core/framework/animation/compiler/sequence/model.rs
  - zircon_runtime/src/core/framework/animation/asset/sequence.rs
  - zircon_runtime/src/core/framework/animation/asset/channel.rs
  - zircon_runtime/src/animation/sequence/compiled.rs
  - zircon_runtime/src/animation/sequence/channel_sample.rs
  - zircon_runtime/src/animation/sequence/conversion.rs
  - zircon_runtime/src/animation/sequence/time.rs
  - zircon_runtime/src/animation/sequence/interpolation.rs
  - zircon_runtime/src/animation/sequence/target.rs
implementation_files:
  - zircon_runtime/src/animation/sequence
plan_sources:
  - user: 2026-09-09 扩展脚本、反射、动画与导航公开接口文档
tests:
  - zircon_runtime/src/core/framework/animation/compiler/sequence/tests.rs
  - zircon_runtime/src/animation/sequence/tests.rs
  - zircon_runtime/src/asset/tests/assets/animation.rs
doc_type: module-detail
---

# Animation Sequence 编译与应用

## 两阶段模型

Sequence 的运行时路径分成两个明确阶段。第一阶段
`compile_animation_sequence` 只处理资产本身，验证时间轴和值域并产生
`AnimationCompiledSequence`；第二阶段 `compile_sequence_for_world` 把这个源产物绑定到
一个具体的 `World`，解析实体并预编译属性 writer。最后，
`apply_compiled_sequence_to_world` 在每帧使用已经绑定的 writer 写回属性。

第一阶段不会排序错误的 key：key 时间必须已经严格递增。它也不会查找实体、加载资源或
修改 World。第二阶段会修改 World 的编译缓存（例如 intern path/field identity），所以其
第一个参数是 `&mut World`，而不是只读引用。

```mermaid
flowchart LR
  D[AnimationSequenceAsset] --> S[compile_animation_sequence]
  S -->|artifact| C[compile_sequence_for_world &mut World]
  C --> X[CompiledAnimationSequence]
  X --> G{is_current_for(world)?}
  G -->|yes| A[apply_compiled_sequence_to_world time looping]
  G -->|no| R[重新绑定]
  A --> Q[CompiledAnimationSequenceApplyStats]
```

## API 形状

```rust
use zircon_runtime::animation::{
    apply_compiled_sequence_to_world, compile_sequence_for_world,
    CompiledAnimationSequenceApplyStats,
};
use zircon_runtime::core::framework::animation::{
    compiler::{
        AnimationCompileSeverity,
        sequence::compile_animation_sequence,
    },
    AnimationSequenceAsset,
};
use zircon_runtime::core::math::Real;
use zircon_runtime::scene::World;

fn evaluate_sequence(
    world: &mut World,
    sequence_asset: &AnimationSequenceAsset,
    time_seconds: Real,
    looping: bool,
) -> Result<CompiledAnimationSequenceApplyStats, Box<dyn std::error::Error>> {
    let source_compilation = compile_animation_sequence(sequence_asset);
    for diagnostic in source_compilation.diagnostics() {
        eprintln!("{}: {}", diagnostic.code(), diagnostic.message());
    }
    if source_compilation
        .diagnostics()
        .iter()
        .any(|diagnostic| diagnostic.severity() == AnimationCompileSeverity::Error)
    {
        return Err(std::io::Error::other("invalid animation sequence").into());
    }
    let source = source_compilation
        .artifact()
        .ok_or_else(|| std::io::Error::other("missing animation source artifact"))?;

    // 这里必须传入可变 World：绑定阶段会建立 typed property writers。
    let compiled = compile_sequence_for_world(world, source)?;
    if !compiled.is_current_for(world) {
        // 应在安全边界丢弃 projection，并重新调用 compile_sequence_for_world。
        return Err(std::io::Error::other("compiled sequence is stale").into());
    }

    Ok(apply_compiled_sequence_to_world(
        world,
        &compiled,
        time_seconds,
        looping,
    )?)
}
```

`AnimationCompileSeverity` 在 `compiler` 模块根部重新导出。若宿主已经在导入阶段拒绝错误
诊断，也可以省略循环并直接读取 `artifact()`。

上例中的 `sequence_asset`、`world`、`time_seconds` 和 `looping` 由宿主提供；它们不是额外
的隐藏 API。`compile_animation_sequence` 的返回值是 `AnimationSequenceCompilation`，应先
检查 `diagnostics()` 和 `artifact()`，再把 artifact 传给 World 编译器。

## 轨道与插值

`AnimationSequenceBindingAsset` 保存 `EntityPath`、可选 `target_id` 和轨道列表；每条
`AnimationSequenceTrackAsset` 保存 `ComponentPropertyPath` 与 `AnimationChannelAsset`。
`AnimationTrackPath` 是用于诊断和索引的稳定组合路径（`entity:path`），不是运行时每帧重新
解析的句柄。source compiler 会保留 `AnimationInterpolationAsset::{Step, Linear, Hermite}`
和已验证的 key；`AnimationCompiledSequenceTrack` 暴露 `property_path()`、
`interpolation()`、`value_kind()` 与 `keys()` 供只读工具使用。

线性和 Hermite 只接受 scalar、vector 或 quaternion 值；Bool/Integer 多 key 通道应使用
`Step`。Quaternion 的 Linear/Hermite 采样会归一化并采用 shortest-path slerp 语义。

## World generation

绑定结果不会只比较一个裸的 World generation。每个 writer 记录所属层级根的
`scene_binding_generation`，并在应用前通过 `CompiledScenePropertyWriter::is_current_for`
检查实体、层级名称和字段 schema 是否仍然匹配。若编译时没有找到目标，路径会进入
`missing_tracks()`；这类结果在 binding-catalog generation 未变化时可以暂时复用，目录一旦
变化就必须重新绑定。实体删除、重命名、reparent、ID 复用或动态组件 schema 变化都应被
视为失效边界。

## 应用统计与错误

`CompiledAnimationSequenceApplyStats` 是 `Copy` 的固定大小结果，公开字段只有：

| 字段 | 含义 |
| --- | --- |
| `applied_tracks` | 采样成功且 writer 写入调用返回 `Ok` 的轨道数；值未改变也计入此项 |
| `missing_tracks` | 编译阶段未解析的轨道，加上应用时 writer/source/target 不可用的轨道数 |

它没有 `skipped_tracks`、`failed_tracks`、duration 或 cache-hit 字段。采样值转换失败（例如
非 finite scalar/vector、零长度 quaternion）会直接返回 `AnimationError`，不会伪造一份部分
成功的 stats；writer 失效则由当前实现计入 `missing_tracks` 并继续处理其他轨道。

## 编辑器与运行时流程

编辑器保存 sequence 后先更新资产 revision，再让运行时丢弃旧 compiled cache。播放头拖拽可以只应用单帧；运行时 tick 应缓存编译结果并传入单调时间。网络回放场景使用确定性 time，禁止以 wall clock 直接驱动。

## 性能建议

- 每个 World/sequence pair 缓存 compiled result。
- 预分配轨道目标数组，避免每帧字符串 split；绑定阶段已经完成 path canonicalization。
- 大量对象按实体批量应用，减少重复 borrow。
- 对缺失轨道做一次诊断去重，不要每帧刷屏。

## 负例

```rust
// 错误：World 拓扑改变后继续使用旧 compiled。
world.remove_entity(entity)?;
apply_compiled_sequence_to_world(&mut world, &compiled, 0.0, false)?;
// 应先检查 is_current_for；为 false 时重新调用 compile_sequence_for_world。
```

## 验收清单

- [ ] source compile 通过 `diagnostics()` 检查后再读取 `artifact()`。
- [ ] world compile 使用 `&mut World`，并在应用前检查 `is_current_for`。
- [ ] 应用前检查 `is_current_for`。
- [ ] missing_tracks 有用户可见诊断。
- [ ] 离散/四元数插值使用正确策略。
- [ ] 资产 revision 变化会失效缓存。

## 源码与测试

- [compiled sequence](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/animation/sequence/compiled.rs)
- [time](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/animation/sequence/time.rs)
- [interpolation](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/animation/sequence/interpolation.rs)
- [sequence tests](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/animation/sequence/tests.rs)

## API 参考

| 符号 | 语义 |
| --- | --- |
| `compile_animation_sequence` | 验证资产并生成 source `AnimationCompiledSequence` |
| `AnimationSequenceCompilation::artifact` | 返回可绑定的 source artifact；有错误诊断时为 `None` |
| `AnimationSequenceCompilation::diagnostics` | 返回稳定 code/severity/element/message 诊断 |
| `compile_sequence_for_world` | 使用 `&mut World` 解析 `target_id`/`EntityPath` 并生成 world-bound projection |
| `CompiledAnimationSequence::missing_tracks` | 返回编译阶段缺失路径切片 |
| `CompiledAnimationSequence::is_current_for` | 校验 binding catalog、实体层级和 writer schema 是否仍可用 |
| `apply_compiled_sequence_to_world` | 按 `time_seconds` 与 `looping` 采样并写入属性 |
| `CompiledAnimationSequenceApplyStats` | 仅提供 `applied_tracks` 与 `missing_tracks` |

## 时间语义

`time_seconds` 使用秒。非 finite 时间会被安全地当作 `0.0`；负时间先钳制到 `0.0`。
`looping=false` 时超过 duration 的时间钳制到 duration；`looping=true` 时只有严格大于
duration 的时间才使用 `rem_euclid(duration)` 回绕，正好位于 duration 的采样仍保留在末帧。
duration 非 finite 或小于等于 epsilon 时采样时间为 `0.0`。调用方应使用单调 simulation
clock，暂停时冻结时间，seek/clock discontinuity 时显式更新自己的插值缓存；该函数本身
不维护跨帧时间状态。

## target 解析

绑定时先尝试把 `target_id` 解析为当前 World 中的 `EntityId`，再尝试把它当作
`EntityPath`；找不到时回退到 binding 自身的 `entity_path`。找到实体后，runtime 取得其
canonical path，并为每条属性调用 typed writer 编译。已知字段（Transform、MeshRenderer、
Camera、Light、Animation*Player）走专用 writer；注册过的动态组件字段走 dynamic writer；
未知字段在绑定阶段返回 `SceneError::UnknownProperty`，不会退回到每帧字符串 visitor。

同一 binding 不允许重复的 property path，source compiler 会发出
`ZR-ANIM-COMP-SEQUENCE-004`。不同 binding 写同一个实体属性时，应用顺序就是 source
bindings/track 的顺序，后写入者覆盖先写入者；需要确定性结果时应在资产校验阶段拒绝这类
冲突。

## 编辑器用法

拖动时间轴只调用 apply，不修改 compiled；保存后递增资产 revision。预览 World 与运行时 World 分离，禁止把预览实体句柄写入持久资产。

## 失败矩阵

| 失败 | 是否继续其他轨道 | 建议 |
| --- | --- | --- |
| path missing | 是 | 统计 missing，构建期阻止发布 |
| type mismatch | 视策略 | 严格模式整体失败 |
| stale generation | 否 | 重新 compile |
| source compile error | 否 | 根据 `AnimationCompileDiagnostic` 修复资产后重试 |
| unknown property / world bind error | 否 | 修正 component/property schema；不要在 apply 阶段兜底解析 |
| world borrow conflict | 否 | 在独占 World 调度阶段调用 |
| non-finite key | 否 | 资产验证阶段拒绝 |

## 性能测量

记录 source compile time、world bind time、apply time、source diagnostics 数量、
`applied_tracks`/`missing_tracks` 和 `is_current_for` 失效次数。长 sequence 应按可见实体
裁剪；不要为不可见对象执行完整属性写回。worker 可以完成 source compile，但 World-bound
compile 和 apply 必须在拥有对应 `World` 的调度阶段执行，避免在线程间传递 `&mut World`。
