---
related_code:
  - zircon_runtime_interface/src/reflect/mod.rs
  - zircon_runtime_interface/src/reflect/type_registration.rs
  - zircon_runtime_interface/src/reflect/read_write.rs
  - zircon_runtime/reflection_macros/src/lib.rs
  - zircon_runtime/src/script/vm/reflection/mod.rs
  - zircon_runtime/src/script/vm/host/reflection_docs/mod.rs
implementation_files:
  - zircon_runtime_interface/src/reflect
  - zircon_runtime/reflection_macros/src
  - zircon_runtime/src/script/vm/reflection
  - zircon_runtime/src/script/vm/host/reflection_docs
plan_sources:
  - user: 2026-09-09 构建 ZirconEngine 详细 Wiki 文档集合
  - docs/zircon_runtime_interface/reflect.md
tests:
  - zircon_runtime_interface/src/reflect/schema_catalog/tests.rs
  - zircon_runtime/src/script/vm/tests/reflection_docs.rs
  - zircon_runtime/reflection_macros/src/tests.rs
doc_type: module-detail
---

# 反射、脚本宏与调用点

## 统一反射模型

`zircon_runtime_interface::reflect` 定义跨 runtime/editor/script 使用的类型描述：`ReflectTypeInfo`、`ReflectTypeKind`、`ReflectTypePath`、`ReflectFieldInfo`、`ReflectTypeRegistration`、`ReflectTypeRole`、`ReflectScriptVisibility` 和 schema/catalog fingerprint。读写通过 `ReflectReadRequest`/`ReflectWriteRequest` 与响应 DTO 表达；值必须经过 `ReflectValueBudget` 与 validation。

类型注册包含序列化策略、字段 identity、editor hint、numeric range 和脚本可见性。schema catalog 使用算法版本和 fingerprint 识别当前注册集合；编辑器或脚本缓存 schema 时必须比较 fingerprint，不要只比较显示名称。

## derive 与 attribute 宏

`zircon_runtime_reflection_macros` 提供三个公开宏，`zircon_runtime` crate root 重新导出：

```rust
use zircon_runtime::{zircon_host_function, zircon_host_module, ZirconScriptType};

#[derive(ZirconScriptType)]
struct Health {
    current: f32,
}

#[zircon_host_function(name = "gameplay.heal")]
fn heal(value: f32) -> f32 { value.max(0.0) }

#[zircon_host_module(name = "gameplay")]
mod gameplay_host {
    // 宏展开为 host module descriptor/注册代码。
}
```

宏展开的细节属于生成代码；调用方应把它视为描述/注册生成器，不在生成文件中加入 bootstrap、World 变更或线程管理。attribute 参数的合法性由 proc-macro 在编译期验证。

## Runtime reflection

`VmReflectionCatalog` 聚合脚本可见 schema，`VmReflectionRegistrySnapshot` 是只读快照，`VmReflectionSchemaInstaller` 把 schema 安装到 host/runtime。World 操作通过 `VmReflectionWorldAccess` 的受限读/写请求执行；写入应在 runtime system/operation 的安全阶段提交，不能从任意 host callback 持有可变 World。

## Rust 调用模式

```rust
use zircon_runtime_interface::reflect::{
    ReflectSchemaCatalog, ReflectSchemaRequest, ReflectTypePath,
};

let path = ReflectTypePath::new("gameplay::Health", "Health")?;
let request = ReflectSchemaRequest::for_type(path.type_path());
let catalog = ReflectSchemaCatalog::try_new(Vec::new())?;
let _ = (request, catalog.fingerprint());
# Ok::<(), Box<dyn std::error::Error>>(())
```

示例表达的是 interface DTO 的构造方向；真正 catalog 来源由 runtime/editor owner 提供。任何用户输入的 type path、field id、object address 和 value 都要处理 parse/budget/validation error。

## 文档生成

Host registry 可用 `ScriptHostInterfaceMarkdownOptions` 生成脚本接口 Markdown。生成输出是当前 registry 快照，不是独立 authority；网站或 IDE 应把其标为生成文档，并回链到源码/API 版本。schema/version 变化时，旧调用点应收到明确 unavailable 或 migration error。

## 限制

- 反射可见不等于可写；`ReflectReadWrite`/script visibility/role 和 owner policy 共同决定权限。
- 类型路径和字段 ID 必须稳定且有字节上限；不要用 debug 名称拼接持久化键。
- interface crate 只放 DTO/验证和错误，不实现具体 World、UI 或 VM 行为。
