---
related_code:
  - zircon_runtime_interface/src/runtime_api.rs
  - zircon_runtime_interface/src/runtime_api
  - zircon_runtime_interface/src/buffer.rs
  - zircon_runtime_interface/src/handles.rs
  - zircon_runtime_interface/src/status.rs
  - zircon_runtime_interface/src/version.rs
  - zircon_runtime_interface/src/profiling.rs
  - zircon_runtime/src/dynamic_api
  - zircon_app/src/entry/runtime_library
  - zircon_editor/src/core/gateway/session
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_app/01-product-host-bootstrap-loop-dynamic-runtime-shutdown-review.md
  - docs/plans/optimize/zircon_runtime/01-core-runtime-lifecycle-registry-review.md
  - docs/plans/optimize/zircon_runtime/02-core-runtime-events-tasks-review.md
reference_engines:
  - dev/godot/core/extension/gdextension_interface.json
  - dev/godot/core/extension/gdextension_interface.schema.json
  - dev/godot/core/extension/gdextension_interface_header_generator.cpp
  - dev/godot/core/extension/gdextension_interface.cpp
  - dev/godot/core/extension/gdextension.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Modules/ModuleInterface.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Modules/ModuleManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Modules/ModuleManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Modules/ModuleManifest.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Modules/ModuleManifest.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Public/ModuleDescriptor.h
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Private/ModuleDescriptor.cpp
  - dev/Fyrox/fyrox-impl/src/plugin/dylib.rs
  - dev/Fyrox/fyrox-dylib/src/lib.rs
  - dev/bevy/crates/bevy_dylib/src/lib.rs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 01 · Runtime DLL ABI、FFI、版本、句柄与跨库所有权工程化差距

## 1. 结论

当前 runtime DLL 边界已经不是临时的单函数样例。`ZrRuntimeApiV6` 集中发布 22 个可选函数指针，App 会保持动态库存活直到 session/frame/buffer owner 释放，Editor gateway 同样用 `Arc` 绑定 library owner；runtime session registry 具备 open/closing 状态、in-flight action 计数、wake entry 禁用、条件变量等待、失败后可重试销毁，并显式拒绝从自己的 wake callback 内销毁。这些基础值得保留。

但该边界尚不能称为稳定、可发布的工程级 ABI。本轮最严重的问题不是“版本号还少”，而是 ABI 身份策略没有闭合：App 对 V6 要求版本和 `size_bytes` 完全相等，表现为 hard cut；握手却没有 runtime/host Build ID、target triple、pointer width、feature set、capability/schema fingerprint，因而结构形状相同但语义不同的两个构建仍会被误接受。反过来，table 使用可选函数和字段可用性工具，却拒绝任何大小不同的 V6。它既没有形成 Unreal 式原子构建集，也没有形成 Godot 式生成接口和可查询能力。

跨库内存所有权还存在直接内存破坏风险。`ZrOwnedByteBuffer` 是 `Clone + Copy`，调用方可复制它；释放函数只检查固定类别 token、指针和 `len <= capacity`，随后使用调用方可修改的 `data/len/capacity` 重建 `Vec::from_raw_parts`。没有逐分配登记、一次性 allocation ID 或幂等释放状态，复制后双重释放、伪造相同 token 或篡改 capacity 都可能触发未定义行为。`ZrByteSlice::as_slice` 又把 null + nonzero length 静默视为空，并在缺少 `isize::MAX` 上限检查时直接调用 `from_raw_parts`；status diagnostics 等路径在结构和预算检查之前就会读取它。

输入、输出和生命周期预算也没有成为 ABI 契约。producer 可先完整序列化 profile、host request、world sync 和 accessibility JSON；`capture_frame` 可按未受限的 `u32` 宽高计算并分配 RGBA buffer；多数事件 payload 没有字节、条目或解码时间上限。App 正在引入 `ForeignOutputState`，为 host/profile/operation/plugin 输出增加结构、字节、条目和事后 decode-time fuse，这是有价值的在途改进，但它发生在 producer 已完成分配之后，且 JSON 解码时间上限只能在阻塞解码完成后检查，不能替代跨边界预算和取消。

本轮登记 5 项 P0、30 项 P1、8 项 P2。建议不在 V6 上继续叠加字段，而是先作一次架构决策：内部 App ↔ runtime DLL 采用带 Build Set Manifest 的严格锁步协议；未来公开 plugin/extension ABI 采用独立的生成式 C SDK、版本/能力协商和兼容窗口。随后以新的 interface family 重做 byte carrier、owned result、status、event、handle、callback 和 cancellation contract，并用编译过的 C/C++ consumer、跨目标布局快照、旧新 DLL 矩阵、sanitizer/fuzz child process 和 OOM/挂起故障测试把 ABI 从 Rust 源码约定提升为可验证产品合同。

## 2. 审查边界与证据

### 2.1 本轮物理范围

| 集合 | 文件 / 物理行 | 证据等级与边界 |
|---|---:|---|
| interface ABI production | 17 / 3,204 | E3：`runtime_api` 全部、buffer/handle/status/version/profiling；不代表整个 interface crate |
| runtime dynamic producer | 44 / 8,791 | E3：导出、table、session、registry、event、surface、frame、operation、world/host/profile 等直接实现；排除 tests 与 `tests.rs` |
| App runtime-library consumer | 9 / 3,226 | E3：library load/table validation/session/frame/output/operation；成文时有其他 Session 修改 |
| Editor gateway consumer | 12 / 1,030 | E3：session owner、owned buffer、world sync、plugin page；成文时 focused production clean |
| interface focused tests | 28 test attributes | E2：16 个 inline，加 ABI safety 9 个、runtime operation 3 个；未运行 |
| immediate producer/App/Editor tests | 166 / 66 / 9 test attributes | E1-E2：交叉阅读关键合同；数量包含 source contract 与在途修改，不能证明二进制兼容 |

指纹算法与前序报告一致：相对路径排序，逐文件 SHA-256，再对 `path<TAB>hash<LF>` 清单取 SHA-256。本轮观察点为：

- interface ABI production：`a7e00a6b8e728e6755651a0909bb2e4f4c8233f53a29ecdc64cedd4e128cf9c0`；
- runtime dynamic production：`e36e2dee32671afe77a2e18198e34ca512c6c7e81703a11d80b522d88780fa9a`；
- App runtime-library production：`31354292fc45a822d4c01e7cfebbd94ed3d0f577d3e0c04598763ba84aba8563`；
- Editor gateway production：`b8a1a0beff501a64ca94daa6843ce2417c169fcfac7281fcee36fc1c4227f83b`。

成文期间 runtime dynamic producer 和 App consumer 正被其他 Session 修改，新增的 App `foreign_output.rs` 也尚未跟踪。App 集合在初始扫描后由 3,014 行、`88aed79f7549a9e9685795164fbf7638b961f384c5a017df7e8246a7d8186179` 漂移到上列复取值；以上指纹只固定审查观察点，不是待实施 baseline。任何修复开始前必须重扫 ABI 定义、producer、App 和 Editor 四端，因此本文标记 `source_recheck_required: true`。

整个 `zircon_runtime_interface` 约 442 个 Rust 文件、50,544 行。本轮只关闭稳定 runtime DLL C ABI 这一纵向切片；plugin ABI、serialization/project/hub、reflection/resource、UI 公共 DTO 仍待后续报告，不能从本文推导为已审。

### 2.2 纵向调用链

本轮不是只看 `repr(C)` 声明，而是逐项追踪：

1. `zircon_runtime_get_api_v6` -> `ZrRuntimeApiV6` -> App/Editor table validation 与函数可用性；
2. session create/destroy -> registry slot/action/wake -> library owner 与 callback trampoline；
3. borrowed bytes -> profile/project/startup/event/input/world/plugin 请求 -> runtime decode；
4. runtime serialization/allocation -> `ZrOwnedByteBuffer` -> App/Editor decode/release；
5. viewport/surface/frame/capture -> out parameter、图像 buffer 与 teardown；
6. operation submit/poll/cancel -> handle、状态 V2、payload 和 terminal release；
7. focused layout/source-contract tests -> 实际缺失的二进制、跨目标和恶意输入 gate。

### 2.3 动态证据边界

本轮为 review-only，没有运行 Cargo、真实 DLL/App/Editor、C/C++ consumer、sanitizer、fuzzer、极端分辨率、跨构建 skew、callback hang 或 unload/reload 测试。静态证据足以确认公开布局、unsafe 入口、free 实现、预算位置、状态机等待和现有测试形态；不能证明任意坏指针在同进程内可被安全探测，也不能证明当前在途改动已通过行为测试。

### 2.4 参考源码给出的边界

- Godot 的 GDExtension 接口由 JSON/schema 和 header generator 维护，C header、文档、since/deprecated、版本查询、proc-address 分发、engine allocator 与 initialization level 属于同一个可发布接口体系。它并不消除扩展 ABI 风险，但证明 ABI 不能只靠 Rust struct 和 source-string tests 维护。
- Unreal 的模块系统不是可直接复制的公共 C ABI；其价值在另一侧：module manifest 携带 Build ID，Module Manager 会拒绝过期或不匹配的模块，并拥有 loading phase、动态重载能力和逆序 shutdown。内部二进制如果选择 lockstep，就必须验证整个构建集身份。
- Fyrox 的动态 Rust plugin 实现明确提示编译器版本间不安全、不建议生产使用，并为热重载复制 library、维持 library owner。它是对“Rust ABI 恰好能加载”不能等于稳定 SDK 的直接反例。
- Bevy 的 `bevy_dylib` 明确服务于增量编译速度，并警告不应作为 release 交付模型。它同样不是第三方 ABI 参考。
- 仓内 Unity Graphics 只包含 graphics package/test 源码，没有 Unity Player/native plugin loader 的权威实现；本文不推断闭源 ABI 行为。

## 3. 已有可保留基础

1. API table 集中导出，避免直接散布几十个平台 symbol；函数指针为 `Option`，给未来能力选择保留了表示空间。
2. App 对 null table、版本、table size 和必需函数做 fail-fast；library owner 被 session/frame 等 wrapper 持有，降低先卸载 DLL 后调用函数指针的风险。
3. Editor gateway 的 foreign buffer wrapper 同样绑定 `Arc` owner，并为 plugin page/world sync 设定了明确字节和条目上限。
4. App 在途 `ForeignOutputState` 已开始统一 success output 的形状校验、release、熔断和指标；这个方向应下沉为 ABI 级合同，而不是删除。
5. runtime session registry 不只是 `HashMap<u64, Session>`：它区分 open/closing，追踪 action/wake，销毁先阻止新动作再等待 drain，并允许失败后重试。
6. wake callback trampoline 在 App 端阻止 Rust panic 穿越 extern C；runtime 也拒绝 callback 自销毁造成的明显死锁。
7. operation V2 已出现 reserved 字段，highlight entity slice 已检查 null/alignment/最大元素，operation submit 会先按 retained budget 检查长度。这些局部防线说明工程化方向已有落点。
8. layout tests 使用 `size_of`/`offset_of` 固定当前 64-bit Rust 布局，source contract 也禁止旧版本静默复活；它们可作为生成 ABI snapshot 的输入，而不是最终 gate。

## 4. 差距清单

### 4.1 P0：必须先修复的内存安全、兼容身份与可用性风险

#### P0-01 · owned buffer 可复制，固定 token 和调用方可写 metadata 允许双重释放或伪造释放

`ZrOwnedByteBuffer` 派生 `Clone, Copy`，公开 `data/len/capacity/owner_token/free`。producer `mem::forget(Vec)` 后使用类别固定 token；free 只比较 token 和 `len <= capacity`，随后按调用方提供的三元组重建 `Vec`。复制 carrier 后调用两次 free、将另一地址配上合法 token、或扩大 capacity，都可能导致 allocator corruption/UB。改为 runtime 内逐分配 opaque allocation ID registry，release 只接收 ID 并实现一次性/幂等状态；或由 host 提供 allocator。不能再让 foreign caller 决定 `Vec::from_raw_parts` 的 capacity，也不能让 owning carrier 为 `Copy`。

#### P0-02 · borrowed slice 与 status diagnostics 在结构验证前构造 Rust slice

`ZrByteSlice::as_slice` 对 null + nonzero length 返回空，掩盖坏 carrier；非空时未检查 `len <= isize::MAX` 就调用 `from_raw_parts`。profile/project/startup、event、IME/file/gamepad、world/plugin 等入口复用该方法；App/Editor 的 `ensure_status` 还会直接读取 diagnostics。应在任何 slice 构造前统一验证 null/len 形状、`isize::MAX`、接口预算和编码要求，并让 status detail 采用有界 owned/inline 载体。任意地址有效性在同进程内不可证明，公开合同还必须说明受信边界或进程隔离策略。

#### P0-03 · frame dimensions 与跨界 payload 缺少 producer-side 预算，可触发 OOM/abort 或拒绝服务

`capture_frame` 接收任意 `u32` 宽高，fallback 通过 `width * height * 4` 计算并分配；整数溢出、超大 reserve 或 allocator abort 不会被 `catch_unwind` 可靠捕获。profile、host request、world sync、accessibility 等输出也可先完整 `serde_json::to_vec`，多数输入没有字节/条目/时间上限。App 的 consumer budget 发生得太晚。所有调用族必须在 producer 工作前协商并执行尺寸、字节、条目、嵌套深度、CPU time 和 retained-memory budget；极端维度必须以确定状态拒绝，不能进入分配。

#### P0-04 · V6 只比较版本与结构大小，却没有构建身份或兼容协商

相同 V6/192-byte shape 的 runtime 与 host 即使来自不同 commit、feature/schema/target 仍会被接受；不同 size 则全部拒绝。函数的语义、JSON schema 或枚举含义变化不会体现在握手中，足以造成静默误解释。内部 DLL 必须验证 Build Set ID、target/data model、feature/schema fingerprint；公开 ABI 则必须返回选中版本与能力。两种策略可共存，但不能继续用一个 `abi_version == 6 && size == 192` 冒充二者。

#### P0-05 · destroy/quiesce 无 deadline/cancellation，挂起 action 或 callback 可永久阻塞退出

registry 的 drain 状态机是正确基础，但等待 action/wake 完成没有 deadline、取消、诊断升级或隔离。一个不返回的 callback、阻塞 renderer/action 或 foreign host 行为可让 session destroy 永久等待，最终阻塞 App/Editor 关闭和 DLL unload。新合同需要 bounded quiesce、取消 token、阶段化诊断和进程级最后防线；callback/action 所属线程必须可识别，超时后不得继续假装安全卸载 library。

### 4.2 P1：发布前必须完成的工程化合同

#### P1-01 · exact table size 与 optional-tail 机制互相矛盾

App 要求 `size_bytes == size_of::<ZrRuntimeApiV6>()`，同时代码又保留字段可用性和 optional function pointer。若 V6 是冻结 hard cut，应删去伪兼容逻辑并依赖 build identity；若允许尾扩展，应接受已知前缀并按 `size_bytes` 访问。必须选定一种可测试策略。

#### P1-02 · `ZrHostApiV1` 的 size 与 callbacks 未参与支持判断

runtime 目前只检查 host ABI version，不验证 table size，也不要求 diagnostics/fetch callback；App 实际传入空 callbacks。Host API 因而是装饰性结构，不是可依赖能力。应定义 required/optional capability、最小 prefix、callback 线程与失败语义。

#### P1-03 · capability 只靠 null function pointer 隐式表达

没有 capability bitset、依赖关系、互斥能力、限制值或 reason。调用方无法区分未编译、平台不支持、权限缺失和临时不可用。握手应返回 capability set 与 limit table，并验证 required host/runtime capabilities。

#### P1-04 · 多数 versioned carrier 没有 `struct_size`、reserved 或 extension chain

除 operation status V2 的 reserved 外，多数 request/event/config 只有 version。任何尾扩展都会迫使完整 hard cut。固定 header 应至少包含 `struct_size`、version、flags/reserved，并规定未知尾字段和未知 flags 的处理。

#### P1-05 · API version 与 ABI version 命名混用

`ZrRuntimeApiV6.abi_version` 实际携带 API version 6，而 session/event 等 carrier 使用 ABI 1/2/3。诊断和文档难以区分 table family、carrier schema 和 wire payload schema。应拆成 `interface_family_version`、`struct_version`、`payload_schema_version`。

#### P1-06 · `usize`、裸指针和 data model 未进入握手

byte length、capacity、IME indices 等含 `usize`，布局依赖 pointer width；当前没有 target triple/endian/pointer width/alignment 标识，也没有 32-bit 布局 gate。内部 lockstep 仍应验证 data model，公共 C ABI 则优先使用固定宽度并显式上限。

#### P1-07 · 没有生成的 C/C++ header、IDL 或非 Rust consumer

当前“C ABI”只存在于 Rust 源码和 Rust test。没有可发布 header、calling convention macro、visibility/export macro、compiler packing assertions、C/C++ example 或 SDK version manifest。应从单一 IDL 生成 Rust `repr(C)`、C header、JSON schema、文档和 layout snapshot。

#### P1-08 · 没有 runtime/host 原子分发 manifest

即使选择内部 strict hard cut，也缺少 Unreal module manifest 类似的 Build ID、configuration、platform、artifact hash 和 dependency set 验证。App 只找到某个 DLL 路径并加载 table。packaging/launcher 必须原子发布并验证同一 build set，拒绝混装旧 DLL。

#### P1-09 · status detail 的所有权、寿命与编码不自描述

`ZrStatus` 携带借用 bytes，但结构本身没有 lifetime、encoding、schema 或截断信息；调用方只能假设函数返回后立即读取 UTF-8。应提供固定 header、raw code、category、detail kind、length cap、correlation ID 和明确 lifetime，必要时用 owned result。

#### P1-10 · 未知 status code 被压缩为通用 Error

consumer 丢失未知 raw status 的数值，前向扩展无法诊断。应保留 raw code，同时映射 known category；未知值必须可记录和透传，不能抹成一个字符串。

#### P1-11 · JSON payload 没有统一 envelope 与 schema identity

profile、host、project、world、plugin、operation 等各自序列化 JSON，部分靠 carrier version、部分靠内容字段，缺少统一 content type/schema version/encoding/compression/limits。语义变化可在 table shape 不变时静默发生。应生成并登记 schema hash，payload 使用 versioned envelope。

#### P1-12 · out parameter 的失败初始化规则不统一

调用方常先写默认值，但 ABI 没有统一规定 callee 在所有失败分支把 handle/buffer/status struct 初始化到可释放状态。每个接口应声明 success/failure ownership transfer，并由 callee 在入口 deterministic initialize out values。

#### P1-13 · 预算属于局部 consumer，而不是接口能力

App/Editor 对部分输出有不同常量，runtime producer 并不知道 host 上限，也没有统一的拒绝状态或实际用量。预算应在握手或 request 中声明，由 producer 在工作前执行；响应记录 produced/truncated/required size。

#### P1-14 · JSON decode time fuse 只能事后报告超时

App 在途实现会在 `serde_json::from_slice` 完成后检查 elapsed；恶意或巨大结构已消耗 CPU。需要 streaming/depth-limited parser、预检 byte/item/depth、可取消 worker 或 typed binary/flat DTO；“完成后发现超过 25 ms”不能当作执行 deadline。

#### P1-15 · 大对象 encode/decode 必须整块驻留

world/profile/accessibility/plugin page 等依赖完整 Vec + 完整 JSON DOM，没有分页 cursor、chunk stream、two-call size query 或 caller-provided destination。大工程状态会产生峰值复制和长尾。应按数据类型选择分页、stream、shared immutable blob 或 host allocator。

#### P1-16 · captured frame 的像素合同不完整

buffer 隐式假设 tightly packed RGBA8；没有 row stride、pixel format、color space、alpha mode、origin、HDR transfer、content rect 或 generation。图像 consumer 无法工程化处理 HDR、padding、backend 格式和异步 readback。应返回 versioned `FrameImageDescriptor` 与 allocation owner。

#### P1-17 · runtime event 是过载的扁平结构

一个 struct 承载约 17 类事件，mouse wheel 把 float bits 塞入 key/scan 字段，window moved 把 i32 坐标转 f32，payload 同时表示 text/path/json/name。应改为固定 event header + 每类 typed payload/union，带 size、timestamp、device/window/viewport identity 和 payload budget。

#### P1-18 · 事件数值与枚举验证不一致

wheel/scale 有有限值检查，但 pointer/touch/motion/gamepad 浮点并非统一拒绝 NaN/Inf；unknown keyboard action 返回 success 并被忽略，gamepad axis 还会截断映射为 Other。所有 raw enum/flags/numeric fields 应由生成 validator 统一处理，unknown policy 必须一致。

#### P1-19 · surface carrier 只覆盖 Win32 与 native-none

当前合同无法表达 Linux Wayland/X11、macOS/CAMetalLayer、Web canvas、Android ANativeWindow、iOS layer 或现代多 surface 能力。surface 不应只追加裸字段；应形成 platform-tagged descriptor、所有权/lifetime、thread affinity 和 recreate contract。

#### P1-20 · viewport 模型固定在默认 handle 1

公开 handle 类型存在，但产品路径大量使用默认 viewport，缺少显式 create/destroy/enumerate、多窗口映射、generation 和 per-surface ownership。多 viewport/split screen/remote/editor preview 无法得到可靠生命周期。

#### P1-21 · callback 没有完整线程、重入和 no-throw 契约

目前只要求 callback 尽快返回，并局部阻止 callback 中销毁自己。没有声明调用线程、并发度、可调用 API 白名单、重入顺序、阻塞上限、panic/C++ exception 禁止和 userdata lifetime。App trampoline 的 `catch_unwind` 不能替代跨语言 SDK 合同。

#### P1-22 · transparent handle 缺少 generation、owner 与 DLL epoch

session/viewport/plugin/operation/subscription 都是裸 `u64`。stale handle、跨 session 误用、DLL reload 后重用和类别混用只能靠各自 map 偶然拒绝。应编码或登记 type、generation、owning session/build epoch，并在诊断中返回 mismatch category。

#### P1-23 · session handle counter 没有 wrap/collision 策略

AtomicU64 `fetch_add` 没有对 0、wrap 和现存 ID collision 的工程化处理。虽然常规运行难以耗尽，稳定 runtime 不能把溢出后的 alias 留为未定义语义。需要 checked allocation、generation 与 exhaustion status。

#### P1-24 · foreign allocation 缺少统一观测和泄漏核算

类别 token 不能回答未释放 allocation 数、bytes、age、owner session、call site 和 teardown leak。新的 allocation registry 应提供 per-session/accounting、high-water mark、leak dump 和关闭 gate，并覆盖 error path。

#### P1-25 · unload/reload 与 build epoch 没有产品状态机

library owner 能防止明显的过早卸载，但没有 begin-drain、reject-new-call、outstanding buffer/callback census、epoch rollover、reload compatibility 和 atomic replacement。Editor hot reload 或产品更新不能只依赖 `Drop` 顺序。

#### P1-26 · API table 仍是人工维护的大型常量/签名集合

runtime API 约有 96 个公开常量/标识符与多组手写函数类型、carrier 和测试，容易出现 producer/consumer/schema/doc 漂移。IDL/codegen 应拥有 numeric namespace、reserved range、name mapping 和 deprecation，人工代码只实现业务 handler。

#### P1-27 · 现有布局测试只证明当前 Rust 64-bit 编译结果

`size_of`/`offset_of` 的 192/96/48 等断言没有经过 C compiler，也没有覆盖 MSVC/Clang/GCC、32-bit、不同 alignment 或 packing 设置。必须生成 C/C++ static assertions，并在目标矩阵编译和运行 smoke consumer。

#### P1-28 · 没有 old host/new runtime 与 new host/old runtime skew 矩阵

测试只验证当前源码 table，没有真实旧 DLL/旧 header artifact。无论采取 strict build-set 还是兼容窗口，都需要保存签名 fixture，验证可接受与必须拒绝的 skew，并校验诊断是结构化且确定的。

#### P1-29 · unsafe carrier 缺少 sanitizer、fuzz 与故障子进程 gate

null/nonzero、misalignment、过大 len/capacity、double release、bad token、unknown enum、深 JSON、callback panic/hang、destroy race 没有 ASan/UBSan/Miri 可覆盖的边界测试。可能 abort/UB 的 case 应在 child process/fuzz harness 隔离执行，CI 验证退出分类。

#### P1-30 · ABI 发布没有兼容性审计和变更流程

当前 source test 会 hard-delete old versions，但没有 ABI diff artifact、review owner、breaking-change checklist、SDK changelog、deprecation window 或 symbol/version policy。应由 CI 对 IDL/layout/symbol/schema snapshot 做差异分类，未经显式 major/interface-family 决策不得发布。

### 4.3 P2：应在主重构中一并收敛的问题

#### P2-01 · window moved 经 f32 中转会损失大坐标精度

原生 i32 坐标被 cast 为 f32，再由 runtime 解释。大桌面/虚拟坐标下不能保证 round-trip。typed event payload 应保留固定宽度整数。

#### P2-02 · 多个 public constructor 接受任意 ABI version

Rust convenience constructor 允许调用方构造不受支持 version，错误延迟到远端。默认 constructor 应只产生当前合法 header，测试坏版本使用显式 unsafe/test builder。

#### P2-03 · `ZrRuntimeTranslatedEventV1` 名称像 C carrier，实际是 Rust wrapper

它没有 `repr(C)`，与真正 ABI struct 命名风格相近，容易被误当成可跨库类型。内部 wrapper 应移到非 ABI namespace 或改名，所有 exported carrier 由 codegen 清单唯一登记。

#### P2-04 · magic default viewport handle `1` 扩散到 consumer

常量虽集中声明，App teardown 等路径仍围绕单一默认 ID 组织。创建 session 应显式返回 viewport set/primary viewport，禁止业务代码猜 ID。

#### P2-05 · unknown input 的处理有“忽略成功”和“报错”两套语义

这会让平台新增值在不同事件族表现不一致。应统一为 retained raw value + known mapping，或明确 strict/lenient flag，并计数诊断。

#### P2-06 · free callback 为 `Option` 但空 free 的成功语义不统一

空 buffer、静态 buffer、malformed nonempty buffer 在不同 wrapper 中可能被接受、报错或泄漏。新 owned result 应让 empty/static/owned 成为显式 storage kind，而不是从 null/free 组合猜测。

#### P2-07 · table 与 carrier 缺少 machine-readable 文档链接

调用方难以从错误定位到准确函数/版本/预算说明。生成 header/doc 应给每个 function/field 稳定 ID、since、ownership、threading、errors 和 example。

#### P2-08 · crate 文档中的“Stable ABI”表述超前于交付物

在没有 header、兼容矩阵、build identity、sanitizer gate 和发布策略时，该表述会给使用方错误承诺。完成新 gate 前应改为“versioned internal FFI contract under development”，并列出支持范围。

## 5. 目标架构

### 5.1 先分离两类二进制边界

| 边界 | 兼容策略 | 必要身份 | 失败策略 |
|---|---|---|---|
| App/Editor ↔ bundled runtime DLL | 原子 lockstep build set；不承诺跨 build 语义兼容 | engine Build Set ID、artifact hash、target/data model、feature/schema fingerprint | 加载前拒绝混装，提供可操作诊断 |
| 第三方 plugin/extension SDK | 独立 interface family；显式版本/能力协商与支持窗口 | SDK family/version、capabilities、host limits、plugin metadata | 选择共同版本或结构化拒绝；不得退回 Rust ABI |

这能解除当前 V6 的矛盾：内部边界可以严格，但严格依据必须是完整 build identity；公开边界可以兼容，但必须由生成式 C contract 拥有。plugin 侧的具体装载、隔离、热重载与权限模型留给后续 `zircon_plugins` 报告。

### 5.2 新握手与生成源

建议引入新的导出族，而不是原地修改 V6：

```text
zr_runtime_get_interface(request, response)

request:
  struct_size, interface_family, min_version, max_version
  host_build_set_id, target/data_model
  required/available_host_capabilities
  byte/item/time/memory limits
  allocator/callback table

response:
  selected_version, runtime_build_set_id, schema_fingerprint
  required_host_capabilities, runtime_capabilities, limits
  proc resolver or frozen selected table
```

单一 IDL 至少生成 Rust/C definitions、C++ wrapper、numeric constants、JSON/binary schema、ownership/thread/error 文档、layout/symbol snapshot 和 validator。不得由 Rust definition、手写 C header、JSON schema 三套真值并行维护。

### 5.3 内存、状态与事件

- `ZrBorrowedBytesV2`：固定宽度 length，统一 shape/limit/encoding validator；任何 Rust slice 构造只能发生在 validator 之后。
- `ZrOwnedResultV2`：immutable ptr/len + opaque allocation ID，`release(id)` 查 registry 并一次性转移状态；caller 不接触 capacity/token。
- `ZrStatusV2`：保留 raw code，带 category/correlation/detail kind/truncated，detail 有固定上限和明确 lifetime。
- `ZrEventHeaderV2`：type、struct size、timestamp、source device、window/viewport/session、flags；payload 为 typed record，不再复用字段。
- `ZrFrameImageV2`：extent、stride、format、color space、alpha/origin、generation、allocation owner。

### 5.4 生命周期与执行合同

每个 API 在生成文档中声明：允许线程、是否并发、是否可重入、callback 期间可调用集合、最大阻塞时间、取消点、out 初始化、ownership transfer 和 error set。session 使用显式 `Created -> Open -> Draining -> Closed/Failed`；drain 关闭新调用、取消可取消工作、等待有界 deadline、核对 callback/action/allocation/buffer 数，再决定是否允许 unload。超时必须保留 library owner 并升级故障，不能释放后继续执行 foreign code。

## 6. 分阶段重构与验收 Gate

### M0 · 先封住 P0，不改变业务能力

1. 给所有 borrowed bytes 增加统一 checked conversion，拒绝 null + nonzero、`len > isize::MAX` 和接口预算；status detail 先做独立有界读取。
2. capture/frame 和所有 JSON 输入输出在 producer 工作前增加 checked arithmetic 与 hard budget；返回明确 `LimitExceeded`。
3. 用 allocation registry + opaque ID 替换 fixed token/free-from-caller-metadata；删除 owning carrier 的 `Copy/Clone`。
4. destroy 增加 deadline/cancellation/diagnostic phases；挂起测试放入 child process。
5. V6 loader 至少临时验证同一 Build Set ID，旧行为只作为明确拒绝的 legacy path。

Gate：double release/forged metadata 不触达 allocator；所有坏 slice 形状在 slice 构造前拒绝；`u32::MAX` frame request 不分配；hung callback/action 能在 deadline 后产生结构化失败且 library 不被误卸载；mixed build DLL 加载失败。

### M1 · 建立 interface IDL、build manifest 与 consumer matrix

1. 定义 runtime internal interface family、Build Set Manifest 和 IDL schema。
2. 生成 Rust/C/C++ definitions、static assertions、docs、schema fingerprint 和 symbol snapshot。
3. 打包 App/runtime/Editor 时写 artifact hash、target、feature 与 build ID；launcher/load path 原子校验。
4. 在 MSVC 与 Clang/GCC consumer 编译 table、carrier 和 callback smoke program，至少覆盖 Windows/Linux x64；若支持 32-bit，纳入对应 data model，否则显式拒绝。

Gate：手写布局变化会由 generated diff 阻止；C/C++ consumer 能创建/销毁最小 session；同 build 可加载，任一 artifact/hash/target/schema 不一致必定拒绝。

### M2 · 切换 typed carrier 与 bounded ownership

1. 发布新的 borrowed/owned/status/event/frame/handle family，不在 V6 尾部偷偷扩展。
2. profile/world/plugin/operation 大对象采用分页、two-call 或 stream，并把 byte/item/depth/time limit 放入 request/handshake。
3. surface/viewport 引入 platform-tagged descriptor、显式 create/destroy 和 generation owner。
4. App `ForeignOutputState` 的 metrics/fuse 保留并改为消费 ABI 返回的真实预算/用量；producer 与 consumer 使用同一 limit source。

- [x] **M2.4 Host-side foreign-output policy convergence and testing slice.** App 与 Editor 已共享 host-only policy owner、session-wide fuse、预算、指标与释放合同；该切片不宣称 producer-side limit source、typed carrier 或分页/stream 已完成。

Gate：所有 success/error/empty/truncated output 的 owner 转移有 property tests；跨 session/stale/epoch handle 被稳定拒绝；HDR/padded frame 可正确解释；unknown event/status raw value 可保留和诊断。

### M3 · 生命周期、兼容矩阵与恶意边界验证

1. 保存 old host/new runtime、new host/old runtime、同 V6 不同 Build ID、能力缺失、schema 变化 fixture。
2. 对 slice/buffer/event/status/operation/callback/destroy 做 fuzz；UB/abort case 在 sanitizer child process 执行。
3. 覆盖 callback panic/C++ exception policy、reentrancy、并发 destroy、double release、leak census、OOM、deep payload、取消和 device/surface loss。
4. 输出 ABI compatibility report、SDK changelog、支持窗口和 breaking-change 审批记录。

Gate：支持矩阵每个组合有明确 accept/reject oracle；ASan/UBSan/fuzz corpus 无已知内存错误；session close 后 action/wake/allocation/operation census 为零；故障注入不会静默成功或卸载仍在执行的 DLL。

## 7. 本轮完成定义与剩余队列

本报告完成的是 runtime DLL stable C ABI 的首轮 E3 静态纵向审查，不是修复完成，也不是整个 `zircon_runtime_interface` 完成。5 P0、30 P1、8 P2 均处于 `pending`；任何“ABI stable”或“可供第三方长期兼容”的声明都必须等待 M0-M3 gate。

后续 interface 队列：

1. plugin ABI/SDK、plugin descriptor、发现/装载/权限/隔离/重载/卸载，与 `zircon_plugins` 纵向联审；
2. serialization/project/hub DTO 的 schema evolution、migration、atomic I/O 与 untrusted input；
3. reflection/resource/public ECS handle 与跨 crate owner；
4. UI/public authoring DTO、版本、未知字段和 Editor/runtime bridge。

成文时没有修改生产代码，没有运行 Cargo 或产品程序。相邻 runtime/App 文件存在其他 Session 的持续修改，实施前必须以新指纹重读 producer 和两个 consumer，尤其核对 `ForeignOutputState`、session teardown、event carrier 与 host output 的最终形态。
