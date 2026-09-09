---
related_code:
  - zircon_runtime_interface/src/serialization/mod.rs
  - zircon_runtime_interface/src/serialization/versioned_schema.rs
  - zircon_runtime_interface/src/serialization/payload_header.rs
  - zircon_runtime_interface/src/serialization/schema_id.rs
  - zircon_runtime_interface/src/serialization/load.rs
  - zircon_runtime_interface/src/serialization/write.rs
  - zircon_runtime_interface/src/serialization/migration/mod.rs
  - zircon_runtime_interface/src/project/mod.rs
implementation_files:
  - zircon_runtime_interface/src/serialization/versioned_schema.rs
  - zircon_runtime_interface/src/serialization/load.rs
  - zircon_runtime_interface/src/serialization/write.rs
  - zircon_runtime_interface/src/serialization/migration/mod.rs
plan_sources:
  - user: 2026-09-09 为 ZirconEngine 构建引擎说明书级 Wiki
  - docs/zircon_runtime_interface/serialization.md
  - docs/zircon_runtime_interface/project.md
tests:
  - zircon_runtime_interface/src/serialization/tests
  - zircon_runtime_interface/src/project/tests
  - tests/fixtures/serialization
doc_type: module-detail
---

# 序列化与版本化契约

## 两类序列化

Zircon 在本分区涉及两种不同的数据寿命：

1. ABI 瞬时消息：world query、host request、plugin event 等 JSON，兼容性由当前 BuildSet、DTO 和调用预算共同保证。
2. 持久化资产/项目数据：必须使用 `serialization` 模块的 schema envelope、版本检查和迁移链。

不要把 `serde_json::to_vec` 的 ABI 消息当作可长期保存的资产格式，也不要为了跨动态库传输而直接发送 Rust/bincode 内存布局。

## VersionedSchema

```rust
pub trait VersionedSchema: Sized {
    const SCHEMA: SchemaId;
    const VERSION: u32;
    fn migrations() -> &'static MigrationChain<Self>;
}
```

每个持久化 payload 必须声明稳定 `SchemaId`、当前版本和完整前向迁移链。envelope 的 `PayloadHeader` 携带 schema id 和 schema version；加载器先验证 header，再解码 payload，并逐步迁移到当前版本。

## 写入示例

```rust
use serde::{Deserialize, Serialize};
use zircon_runtime_interface::serialization::{
    write_versioned, Format, MigrationChain, SchemaId, VersionedSchema,
};

#[derive(Debug, Serialize, Deserialize)]
struct GameSettings {
    volume: f32,
}

impl VersionedSchema for GameSettings {
    const SCHEMA: SchemaId = SchemaId::new("game.settings");
    const VERSION: u32 = 1;

    fn migrations() -> &'static MigrationChain<Self> {
        static CHAIN: MigrationChain<GameSettings> = MigrationChain::new(&[]);
        &CHAIN
    }
}

let bytes = write_versioned(&GameSettings { volume: 0.8 }, Format::Text)?;
```

具体 `SchemaId` 构造形式应以当前 API 为准；新增 schema 前应检查仓库中的命名约定和重复 ID 测试。

文本写入是 canonical pretty JSON，并且只带一个结尾换行。`write_versioned_text_to` 流式写入 caller 提供的 sink，不为写入目的额外构造完整文档。二进制格式编码同样先建立当前 header 和受控 payload。

## 读取与迁移

```rust
use zircon_runtime_interface::serialization::{load_versioned, Format};

let loaded = load_versioned::<GameSettings>(&bytes, Format::Text)?;
let settings = loaded.value;
```

`Loaded<T>` 同时公开 `value` 和 `migrated_from: Option<u32>`；后者可用于提示用户资源已从旧版本迁移。稳定规则是：

- schema id 不匹配立即失败；
- future version 不可猜测，必须失败；
- older version 只有在连续 migration chain 完整时才可加载；
- legacy schema-zero 只能通过显式 `load_versioned_legacy_schema_zero` 路径进入，不能成为默认兼容分支；
- 迁移结果仍需通过当前类型反序列化与业务验证。

## 规范化和安全规则

写入器拒绝 NaN 和正/负无穷，避免文本/二进制格式产生不一致含义。`serde_json::RawValue` 不属于 canonical versioned payload domain。输出受 `SerializationBudget` 限制，超出文档大小、节点、字符串等资源上限会得到 typed error，而不是部分写入后假装成功。

项目路径使用 `RelPath`：反斜杠转 `/`，重复分隔符合并，拒绝绝对路径、盘符/UNC、空路径以及 `.`、`..`。`AssetRef` 使用 `{ guid, path_hint, sub }`，未知字段和非法 subasset path 会被拒绝。

## 错误处理

| 阶段 | 主要错误 | 调用者处理 |
| --- | --- | --- |
| header | schema 不匹配、未来版本、非法 schema id | 停止加载并报告资源身份 |
| decode | JSON/二进制语法、深度或大小超限 | 保留原文件，不写回覆盖 |
| migration | 缺失步骤、步骤失败 | 报告源/目标版本和失败步骤 |
| validation | 非有限浮点、路径或业务不变量失败 | 要求修复资源或使用专用迁移工具 |
| write | sink I/O、预算、编码失败 | 使用临时文件/事务写，成功后再替换目标 |

## 当前状态

- 已实现：文本与二进制 envelope、canonical 写入、预算、typed error、连续迁移链。
- 部分实现：具体资产 schema 的 migration 完整度由各模块负责；不能因基础设施存在就假定所有历史资产均可迁移。
- 规划约束：旧字段 alias、静默默认和“尽力读取未来版本”不属于兼容策略。
