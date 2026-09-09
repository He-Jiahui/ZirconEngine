---
related_code:
  - zircon_reflect_derive/src/lib.rs
  - zircon_runtime/reflection_macros/src/derive_type.rs
  - zircon_runtime/src/script/vm/reflection/catalog.rs
  - zircon_runtime/src/script/vm/reflection/error.rs
implementation_files:
  - zircon_reflect_derive/src
  - zircon_runtime/reflection_macros/src
  - zircon_runtime/src/script/vm/reflection
plan_sources:
  - user: 2026-09-09 扩展脚本、反射、动画与导航公开接口文档
tests:
  - zircon_reflect_derive/src/tests.rs
  - zircon_runtime/src/script/vm/tests/reflection_docs.rs
  - zircon_runtime/tests/runtime_environment_reflection_probe_contract.rs
doc_type: module-detail
---

# 反射 derive、schema 与 Catalog

## 目标

反射系统把 Rust 类型投影为可被脚本、编辑器和序列化层消费的 `ReflectTypeRegistration`。类型注册不是全局可变 HashMap，而是带 owner、generation、revision 的候选快照；这样热重载可以原子替换一组类型。

```mermaid
flowchart TB
  D[#[derive(ZrReflect)]] --> R[ReflectTypeRegistration]
  R --> S[VmReflectionSchema]
  S --> P[prepare_optional_generation]
  P --> V[validate existing Worlds]
  V --> C[commit_prepared]
  C --> W[World::sync_vm_types]
```

## derive 宏

`zircon_reflect_derive::ZrReflect` 生成类型路径、字段描述、读写访问器。脚本类型使用 `zircon_runtime/reflection_macros` 中的 `ZirconScriptType`；host 函数和模块分别使用 `#[zircon_host_function]`、`#[zircon_host_module]`。

```rust
#[derive(ZrReflect)]
#[zr_reflect(plugin = "gameplay")]
pub struct Health {
    pub current: f32,
    pub maximum: f32,
}
```

属性会影响插件 owner、公开字段、默认值和脚本可见性。字段应保持确定顺序；重命名会改变 schema 身份，需提供迁移策略。

## `VmReflectionCatalog` API

- `Default::default()`：包含内建类型的 revision 0 catalog。
- `current_snapshot()`：返回 `VmReflectionRegistrySnapshot` 的不可变副本。
- `revision()`：读取当前提交 revision。
- `apply_to_world(&mut World)`：把当前注册同步到新 World。
- `world_runtime_extension_plan()`：构造 `script.vm.reflection.catalog` 扩展。

快照 API：`registry()`、`revision()`、`current_revision()`、`is_current()`、`can_resolve_names()`。`is_current=false` 时，已编译 dense slot 不能用于运行时 dispatch；`can_resolve_names=true` 仅表示准备中的 candidate 仍可用于装载期名称解析。

## 提交协议

1. 以 slot/generation/owner 构造 candidate。
2. `validate_package_owner` 确认 type path 的 plugin id 与 owner 一致。
3. 校验现有 Worlds 是否能同步新注册。
4. `commit_prepared` 检查 `base_epoch`，防止并发候选覆盖最新提交。
5. 在绑定 Core 时调用 `sync_vm_types_atomically`，然后更新 snapshot/revision。

错误包括 `ForeignPreparedGeneration`、`PreparedGenerationStale`、`SlotOwnerConflict`、`GenerationRegression`、`GenerationConflict`、`TypePathOwnedByAnotherSlot`、`PackageOwnerMismatch`、`RevisionExhausted` 等。错误发生时旧 snapshot 保持有效。

## 字段读写与 schema 规则

反射字段访问必须区分只读、可变写入和缺失字段。脚本传入的 `ReflectedValue` 需要做类型、有限浮点和容器长度校验；不能通过 `Any` downcast 绕过 schema。动态组件的 backing 标记为 `VmTypeBacking::DynamicComponent`，因此卸载插件时必须先撤销组件实例再删除注册。

## 对照

- Unreal `UProperty` 依赖类默认对象；Zircon revision 快照避免在 World 之间共享可变注册表。
- Godot ClassDB 用字符串查找；Zircon 同时保留 canonical path 与 dense registration，兼顾工具可读性和运行时速度。
- Bevy `Reflect` 偏向类型能力组合；Zircon 额外约束插件 owner/generation，服务于动态脚本包。

## 负例与恢复

| 场景 | 结果 |
| --- | --- |
| 新类型 path 属于另一个 slot | 返回 `TypePathOwnedByAnotherSlot`，旧类型继续工作 |
| 使用过期 candidate commit | `PreparedGenerationStale`，重新读取 catalog 后再 prepare |
| 同 generation 内容改变 | `GenerationConflict`，递增 generation |
| owner 与 type path 不一致 | `PackageOwnerMismatch`，拒绝整个 package |
| 当前 World 无法同步 | commit 不发布，保留旧 snapshot |

## 验收清单

- [ ] 每个动态类型声明 plugin owner。
- [ ] reload 递增 generation，不复用旧 dense slot。
- [ ] 所有 World 在 commit 前通过同步校验。
- [ ] snapshot 的 `is_current` 在 dispatch 前检查。
- [ ] 覆盖 stale/foreign/owner conflict 测试。

## 源码与测试

- [derive crate](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_reflect_derive/src/lib.rs)
- [script macros](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/reflection_macros/src/lib.rs)
- [catalog](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/script/vm/reflection/catalog.rs)
- [reflection probe](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/tests/runtime_environment_reflection_probe_contract.rs)

## 注册字段矩阵

| 字段 | 来源 | 用途 | 稳定性 |
| --- | --- | --- | --- |
| type path | derive/manifest | canonical lookup | 不可随意改名 |
| plugin id | attribute/package | owner 校验 | 必须匹配 |
| type hash | schema | 迁移快速判断 | 变化需迁移 |
| field id | derive | 读写/迁移 | 删除需记录 |
| backing | catalog | DynamicComponent/内建 | 影响卸载 |
| revision | catalog | snapshot freshness | 单调递增 |
| generation | slot | reload ordering | 不可回退 |

## 读写流程

```text
脚本 value
  -> type path resolve
  -> field id lookup
  -> ReflectedValue conversion
  -> finite/length validation
  -> component read/write
  -> receipt or VmReflectionError
```

读操作可以使用 immutable snapshot；写操作必须在 World 所有权阶段执行。字段不存在返回明确错误；不要把未知字段默认为零值再写回。

## snapshot 语义

`VmReflectionRegistrySnapshot::registry` 只读；`current_revision` 反映最新提交；`is_current` 反映 candidate epoch；`can_resolve_names` 允许装载期解析但不代表可 dispatch。调用方应把 snapshot 与使用它编译出的 callback/schema 一起缓存，并在下一帧检查 freshness。

## 并发提交测试

两个线程同时 prepare 时，只有 base epoch 匹配者可 commit；另一个必须收到 `PreparedGenerationStale`。测试还应验证 foreign catalog 的 prepared generation 返回 `ForeignPreparedGeneration`，而不是覆盖本 catalog。

## World 同步边界

catalog 绑定 Core 后，提交会先让 LevelManager 验证每个现有 World，再原子更新 registrations。某个 World 无法同步时不发布 candidate。新建 World 通过 `world_runtime_extension_plan` 自动安装当前 catalog；手工 World 应显式调用 `apply_to_world`。

## 最佳实践

- 将 schema 作为插件构建产物版本化。
- 在 CI 中生成 reflection docs 并检查 type path 变化。
- 对可选字段提供默认值和迁移测试。
- 把动态组件与内建组件分开命名空间。
- 记录 catalog revision，不记录内存地址。

## 负例测试

```rust
assert!(matches!(publish_result,
    Err(VmReflectionError::PackageOwnerMismatch { .. })));
```

还应覆盖 generation regression、duplicate type path、同 generation 内容冲突、revision overflow 防护和现有 World 校验失败。

## API 使用顺序

```text
catalog.current_snapshot()
  -> snapshot.registry().type_path()
  -> resolve field/schema
  -> validate value
  -> World write at owned schedule point
```

新插件不要直接调用内部 `publish_generation`（测试辅助接口）；生产路径通过 manager 的 package activation 触发 prepare/commit。

## 文档与工具

reflection docs 工具应输出 type path、plugin owner、fields、read/write 权限和 revision。工具读取 snapshot 时显示 `is_current`；候选 revision 不得冒充已提交 API。对比两个 revision 时按 canonical path 排序，避免 HashMap 顺序造成噪声。

## 兼容性门禁

发布前执行旧 blob 回放、旧 World 同步和脚本读写 smoke test。任何 owner/type/generation conflict 都视为版本门禁失败。
