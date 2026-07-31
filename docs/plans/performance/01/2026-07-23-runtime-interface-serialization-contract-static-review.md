---
related_code:
  - zircon_runtime_interface/src/serialization
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor/11-serialization-and-versioning.md
  - docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md
reference_sources:
  - dev/Fyrox/fyrox-core/src/visitor/mod.rs
  - dev/godot/core/io/config_file.cpp
tests:
  - zircon_runtime_interface/src/serialization/tests
  - current-source Windows serialization tests pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime interface serialization 性能静态审查（2026-07-23）

## 范围与覆盖

`zircon_runtime_interface/src/serialization/**` 当前物理源 **42/42** 个 Rust 文件、**3,598** 行已逐文件阅读，含 text/binary wire、migration chain、typed load/write与测试。2026-07-23首次记录后，外部owner更新10个tracked文件并新增foreign untracked `text/read.rs`；本轮已对这11个文件按当前hash只读复核，不吸收、不修改源码。现有测试共 **44** 条，覆盖text/binary roundtrip、binary v1 golden、格式/版本/错误优先级、body/depth/node/string/container边界、duplicate key、non-finite float、migration table完整性和canonical text。

## 性能结论

- 外部owner已部分实现 **PERF-MVP-570**：current text writer不再构造`serde_json::Value`，而是借用payload进入`CanonicalTextSerializer`并在同一serde遍历检查finite值；current reader用borrowed `RawValue`后直接decode `T`。这减少了完整Value owner，但不是动态验收完成。
- `inspect_text()`为区分legacy与严格envelope，当前依次解析whole document probe、envelope header probe、header signature、whole strict document、strict envelope，之后current payload再typed decode；同一bytes/envelope/header最多约6段JSON遍历。每次load还线性验证同一静态`MigrationChain`。测试只用源码字符串确认direct typed path，没有parse-pass/byte-visit counter；570继续open。
- `CanonicalTextSerializer`为每个scalar生成`String`，array/tuple持`Vec<String>`后`join+format`，object先收集`Vec<(String,String)>`、转`BTreeMap`排序，再为entry/whole object继续`format`；每层祖先都重新物化完整子树文本。Value DOM虽删除，但深/大payload仍可能出现O(serialized bytes × depth)级copy与高峰值RSS，且`write.rs`已膨胀到1,061行。最终实现须使用单一bounded byte sink/chunk owner并按模块owner拆分，不能把“没有Value”误报为streaming完成。
- text format没有与binary对等的输入body、node、container、string或输出byte硬上限。serde JSON默认深度限制只覆盖嵌套，不限制宽对象/数组与大String；项目、偏好、场景或插件输入可在typed失败前放大CPU/RSS。570必须先于DOM分配执法统一hard budget。
- binary write同样先 guard遍历、构造JSON Value和执行冗余canonicalize，再把Value转为flat `BinaryValue`，bincode先产body Vec，最后新建prefix Vec并复制body。binary read先构造flat nodes，再重建JSON Value，migration后再typed decode；current-version仍有三种完整表示。新增 **PERF-MVP-571**。
- binary安全边界是正向基线：wire前缀先校验，body 64 MiB上限在bincode前生效；flat解码迭代执行128深度、2M nodes、1M container entries与16 MiB string上限，future schema在payload value-domain decode前失败。571只减少表示和复制，不放宽这些门或改变 v1 golden。
- `SchemaId`静态声明借用 `Cow<'static,str>`，只在wire decode拥有正文；typed errors主要在失败路径分配上下文String。没有帧级新热点，不另立项。

## PERF-MVP-570 设计

1. text reader在JSON parse前检查hard input bytes，并用一次严格envelope seed取得header与borrowed payload slice；schema/future/chain验证后，current-version直接从payload slice decode `T`。合法current输入完整document/envelope traversal≤1、payload typed traversal≤1；只有无壳v0或旧版本才物化单一`serde_json::Value`并执行migration。
2. `MigrationChain`发布静态、一次验证的 descriptor/shape result；每次load仍观察同一成功/typed错误，但stable schema不再O(V)重扫和重建schema错误String。不得以跳过current chain校验破坏现有负例。
3. writer沿用finite-aware单serde遍历，但把scalar/entry/chunk直接写入一个受限输出owner；需要排序的object只持有界key与entry chunk，不在每个祖先重新`join/format`完整子树。每个最终payload byte的copy与depth解耦，output limit在大`String`形成前生效；按serializer/error/map-key/compound/output owner拆分1,061行单文件。
4. text统一限制input/output bytes、depth、nodes、container entries、single/total string bytes；requested/effective limit和具体拒绝原因可观测。边界必须在完整DOM/输出String形成前生效。

## PERF-MVP-571 设计

1. binary writer用有界 canonical Serde serializer直接构建flat nodes并内联finite/size/depth检查，删除JSON Value和canonicalize；object/struct keys按现有canonical顺序输出，wire v1 node order与golden bytes不变。
2. 在已验证header/chain且source=current时，flat node deserializer直接驱动 `T::deserialize`；只有旧版本才把nodes转JSON Value供migration。duplicate key、non-finite、incomplete/multiple root错误仍在typed decode前报告。
3. 预留prefix后通过bincode writer直接写最终Vec并限制body长度，删除body Vec→final Vec复制；失败丢弃候选Vec，不发布部分wire。decode的node capacity还必须受body-derived byte预算约束，峰值RSS不能只以node count表示。
4. binary/text共享schema descriptor、migration和typed diagnostics，不引入第二条兼容reader或更改magic、endianness、varint、trailing-byte与header-first规则。

## 参考引擎对照

- Fyrox Visitor binary writer接受任意 `Write`，文件路径直接用 `BufWriter`写同一binary stream；Zircon虽需先返回 `Vec<u8>` ABI，但也可把prefix/body写入同一最终缓冲，不必先生成第二个body Vec。
- Godot `ConfigFile::_internal_save`直接向 `FileAccess`逐项输出，说明持久层可面向writer而非先建完整文件String；其每个Variant仍先产生临时String，Zircon只采用direct-writer边界，不照搬逐值String分配。

## 动态验收

1. current-source 44条interface合同，以及DynamicScene、reflected JSON、preferences、export preset真实consumer；binary v1 golden与所有typed错误匹配逐项不变。
2. payload 1 KiB/1 MiB/64 MiB、nodes/containers 1/1k/1M、depth 1/64/128/129、versions current/0/long chain：记录whole/envelope/header/payload parse passes与bytes visited、DOM/node owners、fragment/String/Vec copied bytes、chain validations、p95/RSS。text current完整envelope traversal≤1、payload typed traversal≤1、payload Value owner=0、每个输出byte copy与depth解耦、shape validate≤1/schema。
3. text bytes/string/node/container max/max+1和malicious wide input：拒绝在完整DOM/String形成前，RSS有硬上限；canonical bytes、single newline、legacy magic detection与错误优先级不变。
4. binary current JSON Value owners=0，body→final copy bytes=0；旧版本仅一个migration Value owner。5k/100k entity DynamicScene与F0/F2 save/load产品trace验证吞吐、peak RSS和字节等价。

current-source Cargo、规模 traversal/allocation counter、consumer hard-cut和F0/F2/F4产品 trace未完成，因此该目录继续保留在 `pending.md`，不进入 `review.md`。
