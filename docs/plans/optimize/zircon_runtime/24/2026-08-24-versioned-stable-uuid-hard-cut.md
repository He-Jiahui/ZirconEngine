---
title: Runtime24 M0 Versioned Stable UUID Hard Cut
category: zircon_runtime
report_id: Runtime24-M0
date: 2026-08-24
baseline_head: 0e2bdaa9d3f6949e351ce4e77ccf1aca9e7032b1
baseline_epoch: 383
session_id: optimize-runtime24-stable-uuid-v1-hard-cut-r1-20260824
implementation_status: implementation_complete
validation_status: managed_cargo_pending
review_status: independent_review_pending
---

# Runtime24 M0 Versioned Stable UUID Hard Cut

## 目标与 current-source 结论

`zircon_runtime_interface::resource::stable_uuid_from_components` 原先把 namespace 与 components 用 `0x1f` 拼成
`String`，再交给标准库 `DefaultHasher` 两次生成 128 bit。标准库不承诺该 hasher 是持久格式；delimiter 也未转义，
所以 `components = ["a", "b"]` 与 `["a\u{1f}b"]` 的 preimage 完全相同。该函数同时拥有
`AssetUuid::from_stable_label`、`ResourceId::from_stable_label/from_locator/from_asset_uuid` 的派生身份。

本切片只拥有持久派生 UUID 算法，不扩到 Runtime24 的 live handle、World owner epoch、slot retirement、Scene
allocator exhaustion 或 ABI handle。父计划保持只读。

## 参考引擎边界

实现前复核了父计划所列 current reference：

- Unreal 将 weak object index/serial 与可解析 object handle 分层，强调 live object owner/serial；
- Bevy 明确 entity bits 只在同一 App 实例有效，generation 不是持久身份；
- Fyrox pool handle 以 index/generation/type 约束同一 pool 内对象；
- Godot ObjectID 与 RID owner 属于不同注册域，RID validator 只证明当前 owner 内有效。

这些实现共同证明 live handle 不能冒充 persistent identity，但没有要求 Zircon 复制其内存布局。Zircon 的派生持久
ID 因而保留 UUID 外形，并以显式版本、domain separation 与稳定 byte framing 定义自己的跨工具合同。

## V1 byte-level schema

算法使用仓库已有 `blake3` 依赖，不增加 Cargo 依赖或修改 dirty `Cargo.lock`：

```text
BLAKE3 derive-key context = UTF-8("zircon stable identity UUID")
preimage =
    u32_be(STABLE_UUID_ALGORITHM_VERSION = 1)
    + u128_be(namespace_utf8_byte_len) + namespace_utf8_bytes
    + u128_be(component_count)
    + repeat(u128_be(component_utf8_byte_len) + component_utf8_bytes)

uuid_bytes = blake3(preimage)[0..16]
uuid_bytes[6].high_nibble = 8
uuid_bytes[8].high_bits = 10b
```

字符串按原始 UTF-8 bytes 处理，不隐式 trim、大小写折叠或 Unicode normalize；这些 canonicalization 必须由每个
namespace 的上游 locator/label contract 明确拥有。固定宽度大端长度和 component count 让所有边界无歧义，并且
不依赖 host pointer width、endianness 或 Rust hasher 实现。

## Hard cut 影响

1. 删除 `DefaultHasher`、joined `String`、双 hash 与 delimiter 编码，不保留旧算法入口、alias 或 fallback。
2. 导出 `STABLE_UUID_ALGORITHM_VERSION`，让 build/schema/diagnostics owner 能记录派生身份算法版本。
3. `AssetUuid::new()`、显式 UUID parse/serde 与项目中已存的显式 UUID 字节不改变。
4. `from_stable_label`、非 memory locator 派生的 `ResourceId`、以及 `from_asset_uuid` 派生值会按 v1 重新生成。
   旧 derived registry/cache 必须重建；不得同时接受旧/new 派生 ID。
5. memory locator 仍生成 v4 UUID，不进入稳定算法。

## RED 与实现证据

测试先于 production 修改写入：

- component framing 测试用旧实现可确定复现 delimiter collision；
- version/variant 测试要求 UUID v8 与 RFC variant，旧实现未设置这些位；
- v1 固定向量把 `zircon-asset-uuid` + `res://materials/hero.zmaterial` 锁定为
  `189d05ad-e595-8f2b-94c0-615f977daa11`，防止平台、工具链或实现重构漂移 byte schema；
- public resource contract 先引用不存在的算法版本常量，并验证同 label 在 Asset/Resource namespace 下不同。

实现用 `blake3::Hasher::new_derive_key` 流式写入 framing；不构造 joined buffer。UUID 只取 digest 前 128 bit，再按
RFC UUID v8/custom 形式设置 version/variant 位。碰撞安全目标仍是 UUID 级 122 有效 hash bits，不把 UUID 当内容
完整性 digest 或安全签名。

## 性能边界

新路径从“一次 joined String 分配 + 两次 DefaultHasher”变为“一次流式 BLAKE3 + 零 joined allocation”。这是正确
schema 的实现结果，不是经过 profiler 证明的优化结论。`PERF-MVP-564` 仍必须以真实 project scan/import 负载量化
locator format、UTF-8 framing、hash 和 allocation；本切片不宣称耗时、功耗或跨引擎优势。

## Managed validation evidence

首个 GREEN ticket `9182f8ce94634cabb8ca96e37b7d20d1` 没有编译或执行测试。其 overlay manifest 错误包含了
未修改、无需 overlay 且没有本 Session attribution 的 `zircon_runtime_interface/Cargo.toml`，协调器以
`validation_copy_overlay_not_owned` 在 materialization 阶段 fail closed。该结果不是算法失败或 GREEN 证据；替代
ticket 只覆盖已归属的 child record、`resource/mod.rs` 与 `stable_uuid.rs`。

替代 ticket `f60313d4357649039f37b136293aa816` 已正确使用三文件 owned overlay，但仍在编译前失败：
validation copy materialization 检测到与本计划无关的 6 个 Runtime74/RHI `AM` 路径发生
`validation_copy_baseline_drift`。因此该 ticket 同样不是 GREEN/算法失败证据；在共享漂移路径完成 ownership
收口前不继续盲目重试。

## 状态

- [x] current source、所有派生 identity consumer、serde 边界与 Cargo 依赖复核。
- [x] Unreal/Bevy/Fyrox/Godot persistent-vs-live identity 边界复核。
- [x] delimiter collision、UUID version/variant 与 public version RED contract 先行。
- [x] 独立 `rustc` 计算并固化 v1 跨平台 UUID exact vector。
- [x] versioned BLAKE3 UUID v8 schema 与 hard cut 实现。
- [x] `Cargo.toml` 与 dirty `Cargo.lock` 保持不变。
- [x] 3 个 Rust 文件 exact `rustfmt --check` 与 scoped `git diff --check` 通过。
- [ ] managed `zircon_runtime_interface` focused tests 实际 GREEN。
- [ ] managed interface package build 与 downstream core-min compile。
- [ ] 独立 reviewer 复核 byte schema、hard cut 影响与测试充分性。
- [ ] coordinator immutable manifest、service commit 与自动 WeCom 量化通知。

在 managed Cargo 与独立复审完成前，本里程碑保持 `implementation_complete / validation_pending`，不得提交。

## Current Runtime24 ownership and static handoff (2026-08-30)

本次 Runtime24 session `root-runtime24-stable-uuid-v1-20260830` 已对本计划目录、该计划文件、
`zircon_runtime_interface/src/resource/stable_uuid.rs` 与 `zircon_runtime_interface/src/resource/mod.rs`
完成精确 scope claim（request `482c5043309049dd91be05f9c835a552`）和 baseline attribution
（request `bec7207691ba4ecf8be7ee0d90b362db`）。当前源码哈希为：

- `stable_uuid.rs`: `2FE5E40A6FB10FC5ECBD4181FB87D9D53A07F385E8CE52D53A6EB5A3AF422734`
- `resource/mod.rs`: `DC3038E4E38AF841423BD35B23784FC8ED27E3B4292860F29CBD5C3D38570633`
- 本计划文件（写入前基线记录）: `234A6B86D936649FADC017BF7432D5C0EDCF2B942BD9F2232DC494CA31137194`

Rust 1.94.1 `rustfmt --edition 2024 --check --config skip_children=true` 与 scoped
`git diff --check` 已通过；只读结构扫描确认版本常量、BLAKE3 derive-key、namespace/component byte framing、UUID
v8 与 RFC variant 位均存在，`DefaultHasher` 与 delimiter-joined 路径均已移除。此记录不改变 managed Cargo
validation、独立 review 或性能矩阵仍 pending 的状态；不构成 GREEN、commit 或 WeCom 性能通知。
