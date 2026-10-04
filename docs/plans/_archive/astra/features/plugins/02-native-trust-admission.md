---
status: in_progress
review_date: 2026-09-05
source_recheck_required: true
plan_sources:
  - docs/plans/astra/optimize/01-review-and-repair.md
  - docs/plans/optimize/zircon_plugins/01-plugin-sdk-package-catalog-distribution-native-abi-review.md
  - docs/plans/optimize/zircon_plugins/20-plugin-sdk-example-native-editor-fixture-test-carrier-artifact-isolation-product-truth-review.md
  - docs/plans/optimize/zircon_plugins/21-plugin-artifact-marketplace-third-party-package-install-update-trust-non-cargo-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/07-script-plugin-runtime-review.md
---

# Native Plugin Trust Admission

## 目标与边界

本计划建立 Native plugin 在任何动态库代码执行前的统一 trust admission。权威调用方必须提供期望 package identity、artifact digest、签名/信任结论、ABI/engine compatibility、目标与 capability 约束；loader 只能验证候选 sidecar 与 DLL bytes，禁止从候选 manifest 复制期望 digest、BuildSet 或 identity。范围包括首次加载、scan/discovery 消费与 hot reload 复用的同一生产 admission，以及拒绝后的 generation 保留。

复用现有 Package Service、signature/trust 与 native artifact manifest 合同。Untrusted、identity mismatch、digest mismatch、capability mismatch 与 manifest compatibility mismatch 在原生库加载前失败并拒绝进程内执行。本阶段主验收 Windows；其他平台因缺少不可替换加载路径而明确拒绝。包含构建期 export authority producer 和当前 Session 精确 lease 的真实 DLL fixtures；Marketplace、Hub、render internals 与 Tooling 实现不在本里程碑内。

## M1 · Authority 与 Admission Contract

1. 盘点现有 native artifact manifest、Package Service verification 与 signature/trust 类型，选定唯一 authority 输入，不用 bool/string 复制身份。
2. 定义不可由候选自证的 expected identity/digest/signer/trust/capability/target contract，并为受信本地首方 artifact 提供显式 authority 构造。
3. Admission 在打开 DLL 之前读取 sidecar、计算 DLL digest、验证 package/module identity、ABI/engine compatibility、目标与 capabilities，并产出 typed accepted/rejected 结果。

## M2 · Initial Load、Scan 与 Hot Reload

1. 首次加载和 scan/discovery 消费统一 admission；任何 `Library::new` 前必须已有 accepted receipt。
2. Hot reload 对候选 generation 使用相同 admission；失败不得调用 entry、不得替换 active library/state/generation。
3. Rejected artifact 进入隔离/诊断路径，错误携带稳定原因和候选路径，但不得把候选字段提升为期望值。

## Combined Acceptance

1. Identity、DLL digest、signer/trust、ABI/engine compatibility、target 与 capability 检查全部在任何 `Library::new` 前完成。
2. Trusted local first-party admission 的 expected values 来自调用方 authority；candidate manifest 与 sidecar 只作为 actual evidence。
3. 被拒 DLL 的 entry side effect 计数保持零；hot reload 拒绝后旧 generation、active library 与可服务状态保持不变。

## Implementation Snapshot

- `NativePluginArtifactAuthority` is the sole in-process admission authority. It defaults to deny-all and accepts only explicit expected package identity, manifest/DLL SHA-256 plus byte length, trusted authority identity, target, module kinds, and capabilities. Admission independently rejects Sample/TestFixture package roles even when a structurally matching authority expectation is supplied; role filtering is not trusted solely to catalog, export, or authority-producer callers.
- Discovery remains available for diagnostics, while every `Library::new` path requires an admission receipt first. Export load and delta-pack hot update use their host authority; project-directory discovery is no longer allowed to capture its own DLL/manifest digests and mint `TrustedLocalFirstParty` authority. Editor project load and Play resolve deny-all until independently verified host policy or build-embedded authority is supplied.
- Rejected candidates are reported before descriptor or entry probing. Existing hot reload retains the active generation because admission occurs before the replacement enters lifecycle teardown.
- Plan 03 implements the runtime Ed25519 verifier using the existing ProductReceipt canonical wire. Its opaque proof is the only signed-package authority constructor; key/registry/receipt expiry is retained and rechecked during every admission. Directly supplied SignedPackage expectations remain denied.
- Windows admission copies and hashes the main DLL and authenticated dependency closure into a fresh directory, retains no-write/no-delete file handles and a directory deletion guard, validates PE architecture and eager imports, rejects delay-import and package export-forwarding policy gaps, recursively admits System32 imports and export-forwarder targets, verifies API-set mappings, and pins trusted system modules before LoadLibraryEx with DLL_LOAD_DIR | SYSTEM32. A preloaded package dependency is reusable only when its load-time path and digest match a still-retained, host-admitted staging generation with immutable file guards; rehashing the current pathname does not authenticate an existing mapped image. The rename-and-replace child-process regression and guarded-generation reuse test are source additions pending managed Windows DLL execution.
- Native-aware directory and ZIP materializers validate selected source package roles and capture manifest/main/dependency digests plus open source handles before modifying output. Every directory replacement and ZIP entry hashes the exact bytes streamed from those handles and must match the captured authority digest; directory files and the complete ZIP publish from same-directory staging only after verification, so an in-flight source mutation cannot replace prior output. A generated file that aliases the reserved `src/zircon_native_authority.json` path through repeated separators or ASCII case changes is rejected before staging either output. The generated executable embeds the metadata and passes authority through export bootstrap into ProductCompositionRequest. Product target mismatch and missing required native registrations reject before Core bootstrap.
- Bare `ExportBuildPlan::materialize`, `write_generated_files`, and `preview_materialize` fail closed for native selections because they lack the package root needed for role/artifact admission; the native-aware materializer retains its separately prevalidated generated-file path. The Editor export preparation rejects selected Sample/TestFixture packages before creating cache directories, synchronizing staging, running native Cargo, or pruning prior staging. Editor's native completion, registration, status, and enablement surfaces exclude these carriers; an enabled but excluded selection receives an explicit status diagnostic. Required NativeDynamic EditorHost selections reject project Ready when the product package, trusted load, or editor registration is missing; optional failures remain visible in native status.
- Real MSVC DLL tests cover positive immutable staging, original build authority generation, signed receipt to DLL admission, tampered main/dependency zero DllMain side effects, unlisted dependency denial, forwarded-export denial, already-loaded same-name spoof denial, and child-process cwd/PATH spoof resistance. Existing Rust native fixture tests now pass authority captured from original build artifacts; hot reload rejection verifies the old echo callback remains usable.
4. Initial load、scan/discovery 与 hot reload 共享生产 admission，无绕过入口。
5. 格式与源码自审由本 Session 完成；DLL/Cargo 测试执行和最终接受只由父 Astra combined validation 负责，目前尚未声称通过。

External inputs: production signed-package trust roots, current revocations/validity policy, expected package release/BuildSet/target/capability bindings, and signed receipts must come from the host installation/account service. No external account/backend is needed for first-party source selected while producing a trusted product binary; a later project selection of DLL bytes is not a build-owned trust root. No network or candidate-provided registry is silently promoted to trust authority.

## 状态与产出记录

| 里程碑 | 范围 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| M1/M2 | Native inventory 校验、独立 trust authority、Runtime/Editor load、Play 激活与开发 hot reload 共用 admission | `in_progress` | — | `NativePluginArtifactAuthority` 默认 deny-all；导出构建期 authority 与 runtime signed proof 验证仍在。2026-09-21 复审发现 Editor 曾从同一候选项目 DLL/manifest 捕获摘要并授予本地 trust，已移除该自我背书；项目/Play 在缺少宿主独立证明时拒绝原生执行，required NativeDynamic EditorHost 阻止 Ready，可选项输出失败诊断。尚需宿主信任根/已验证 proof 的产品接线，随后执行受管 Cargo、真实 MSVC DLL/side-effect 及性能验收；`E:/Git/zr_vm` 脏依赖仍阻塞原生验收。 |
