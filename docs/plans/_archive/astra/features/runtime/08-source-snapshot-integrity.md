---
status: in_progress
plan_sources:
  - docs/plans/astra/features/01-product-correctness.md
  - docs/plans/astra/features/runtime/05-module-wiring-repair.md
---

# 资源源快照一致性

## 触发条件与根因

本里程碑实现 ASSET-A4 保存边界的对称约束，并完成 ASSET-A2 中 glTF 外部源的
已授权快照消费。当前 `ProjectManifest::save` 在校验后先完整序列化，再直接进入原子
写事务，未在替换旧文件前执行与加载端一致的 `MAX_PROJECT_MANIFEST_BYTES` 检查。
当前 glTF 路径先用 `AuxiliarySourceResolver` 验证 URI，但丢弃解析出的路径，随后
`gltf` 库按原 URI 再次打开文件；这既扩大了验证/使用间竞态，也绕过了
`AssetImportContext` 中优先级更高的不可变辅助源快照。

全量 generation 的 compound source 还会为 digest 读取所有成员，随后 importer 再次
读取相同文件。ASSET-A3 要求 generation 在 restore 判断前冻结成员关系和内容，用流式
digest 绑定主源、成员路径与成员 bytes，并复用于 importer。累计输入采用 artifact store
既有的 2 GiB raw payload 上限，成员数采用 65,536 的运行时大集合上限；读取使用有界
reader，避免元数据长度导致超大预分配。ASSET-A1 仍指加载端既有的 manifest 有界读取。

## 修改范围

- `project/manifest/save.rs`：在原子写前对序列化后的 UTF-8 字节执行同一 manifest
  上限，返回既有 typed `DocumentTooLarge`，确保超限失败不触碰旧文件。
- `importer/ingest/gltf_decode.rs` 与 `auxiliary_source.rs`：外部 buffer/image URI 在
  restore 判断前解析、准入并有界读取成快照，decoder 只消费该代 bytes。外部图片从
  解码后的路径、MIME 或签名识别 PNG/JPEG/WebP。`gltf_decode/{sources,buffers,images}`
  分别持有一次导入的源缓存、缓冲区准入和图像解码；直接导入没有预置快照时，同一
  lexical 引用也复用首次打开的 bytes。GLB、data URI、meshopt fallback 与外部源的
  每份 padded buffer，以及所有来源的 decoded pixels，共享累计 2 GiB 解码预算。
  声明长度在读取前预检，实际长度在克隆/分配前补充准入；外部 buffer 读取使用输入
  与剩余解码预算的较小值。data URI 保留 gltf 的 payload/optional-padding 语义，
  使用 base64 库的切片 API 写入已准入的 fallible buffer；图片 data URI 的临时 encoded
  bytes 与像素输出共同检查峰值。全部 image buffer view 使用 checked range，再交
  image decoder。`ProjectPaths` 的无 I/O lexical identity 复用平台大小写排序，让 Windows
  case aliases 在快照生产与直接导入缓存中只读取和计费一次。
- `project/manager/scan_and_import/{sources,full_generation,targeted}.rs`：generation 在读取
  source 时建立主源和成员快照，把 deterministic digest、mtime 与 importer 输入绑定到
  同一代 bytes；shader descriptor 声明的 package 外 WGSL 也进入快照和 digest。importer
  在有快照时从快照键冻结 descriptor/implicit WGSL 成员关系，避免二次目录扫描。

不修改 manifest root validation 测试或合同；该区域由 `astra-asset-roots-20260905`
拥有。保留工作树已有 template receipt、路径 resolver 与优化测试改动。

## 验收

- exact-limit manifest 可保存，limit+1 返回 `ProjectManifestSummaryError::DocumentTooLarge`；
  目标存在时超限失败保持旧字节不变，且原子写 fault 路径的既有语义不变。
- glTF 外部 buffer 与 image 使用准入路径对应的 context snapshot；snapshot 与磁盘不同时
  snapshot 胜出，缺少 snapshot 时合法文件仍可读取；embedded/data URI 与 typed parse
  diagnostics 保持有效。
- 解码预算回归使用小注入上限，覆盖单个与累计 fallback、重复 URI 的实际 padded 副本、
  data URI 声明/实际长度差异、重复 PNG/WebP 像素、buffer 与 image 共享上限以及非法
  image view 范围无 panic。直接源缓存读取后删除原文件，后续别名引用仍使用首次 bytes。
  图像 metadata 的 checked width/height/channel 乘积在分配输出前准入；RGB WebP 转换
  同时计算原 RGB 与目标 RGBA 峰值。输出使用 fallible reserve；解码器显式保留既有
  512 MiB 临时分配上限，该上限受 image 库内部 best-effort 行为约束，不声明整个进程
  RSS 的硬上限。实际 Cargo 和 compound 峰值测量仍待受管批次。
- 全量与 targeted generation 的 importer context 获得 restore 判断前形成的成员快照，
  source digest 与 imported bytes 属于同一代；外部 glTF 或 package 外 WGSL 的单独变化
  会改变 digest 并触发 reimport。累计 2 GiB 和 65,536 成员边界返回确定错误。
- 本实现 session 只执行 `rustfmt --check`、source guard、`git diff --check` 和文件哈希；
  Cargo/build/test 交由父 session 统一批量验证。

## 状态与产出记录

| 里程碑 | 范围 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| source snapshot integrity | manifest save、glTF/shader snapshot producer 与 consumer | implementation_review_pending | - | 已增加源级回归；未执行 Cargo 验证 |

当前实现已将准入、打开、身份确认和读取绑定到同一文件句柄：Windows 使用不跟随 reparse
打开、最终句柄路径和文件身份比较；Linux/Android 使用 `/proc/self/fd`，macOS/iOS 使用
`F_GETPATH`。`auxiliary_source.rs` 的读取 API 与 `astra-runtime-wiring-20260905` 的外部
基线保持兼容，当前 lease 已明确归属本 session。ProjectManager 端到端 reimport/race 验证、
大 compound 峰值分配证据和 preflight 失败对原有逐资源失败处理的影响仍交由父 session 的
统一 Cargo 批次审查；本 session 未执行 Cargo。
所有 direct Cargo 尝试仅等待 build-directory lock 后终止，没有测试通过证据。

## Review Candidate 2026-09-05

Configured project/package root authority is carried by `AssetImportSource` into primary,
compound, glTF and shader reads. Lexical snapshot keys preserve admitted URI spelling;
Windows frozen lookup handles case/verbatim aliases without a filesystem probe. Declared
auxiliary references share the 65,536 member budget. Review is pending on this replacement
candidate; Cargo validation has not run. Exact auxiliary lease receipt:
`aa3a52cd2c6745bda415052b5bdddf2d` (acquired, no conflicts).

New regression: `gltf_auxiliary_count_limit_is_checked_before_opening_files`. The product
deletion-race regression uses an uppercase buffer URI on Windows with a lowercase filename.

```text
E:/Git/ZirconEngine/zircon_runtime/src/asset/project/manifest/save.rs 8A251474650C8ADD87FD927E13F567FBB29E2DE947B59CC6F1C86617DAA85967
E:/Git/ZirconEngine/zircon_runtime/src/asset/importer/contract.rs 21F7983184D033A2EAA83B48796D0B0454A78A825D7C2F1BDC360B562DC4498A
E:/Git/ZirconEngine/zircon_runtime/src/asset/importer/mod.rs 8D1E74DD55108FE6E44DDA8A0E55E5798E5A79ACF26B9AC1BF567367DCE7B302
E:/Git/ZirconEngine/zircon_runtime/src/asset/importer/ingest/mod.rs 4BD66FCC252BC3F315F97F68D3D3D4D32AF7DD9CFB1BF9B29D720C124A55B6AC
E:/Git/ZirconEngine/zircon_runtime/src/asset/importer/ingest/auxiliary_source.rs 166C9C4202C3D11AC0FA1F4112216D5D63986B52FCA11A94A74326F80CBA2002
E:/Git/ZirconEngine/zircon_runtime/src/asset/importer/ingest/auxiliary_source/opened_snapshot.rs 8B702E8BED69BF43207F2FAD25753528DCFC7F794512F7CD6FA73DFF1E031275
E:/Git/ZirconEngine/zircon_runtime/src/asset/importer/ingest/gltf_decode.rs 1396D21E2FF9FEDECB6D66CDF53259398832ED02656122F4C02DDB187A9B649F
E:/Git/ZirconEngine/zircon_runtime/src/asset/importer/ingest/import_shader_package.rs DE8BD49CEC226FCB0B9566DF0D2BC5AF67890A9F0E44B4807A79BFF6AD1D8318
E:/Git/ZirconEngine/zircon_runtime/src/asset/project/manager/scan_and_import/sources.rs 8740C89A58E4C69814554299997D9957EECCFFABE1FB1FD703FD8546643B77AC
E:/Git/ZirconEngine/zircon_runtime/src/asset/project/manager/scan_and_import/full_generation.rs 79350A96A8F4C882A473751C82A7E116772D26D5BF5A816400484D9ED3A384A1
E:/Git/ZirconEngine/zircon_runtime/src/asset/project/manager/scan_and_import/full_generation/source_snapshot_tests.rs EDBC675A3E415D39CBBC1E0C5BB1D48C2C37ABF9D20391545D54956CA6FCB5AB
E:/Git/ZirconEngine/zircon_runtime/src/asset/project/manager/scan_and_import/targeted.rs 439D10154D36D306E2A40985E663AE2D9AD5CC185B5D6AB6D126C9DA4373B1C2
```

## ASSET-A3 sidecar parse admission (2026-09-26)

One `.zmeta` document now has an explicit 64 MiB serialized byte limit. The 2 GiB cumulative
source payload limit is too large for a TOML parse allocation. This is a product admission policy:
compound sources may contain 65,536 members and `ResourceLocator` has no URI byte-length cap, so
the 64 MiB value cannot be inferred from the existing member count. A previously parseable sidecar
larger than 64 MiB will now be rejected. The reader checks file metadata before allocating and
reads at most limit plus one byte if the file grows. In-memory parse and serialization before
atomic save enforce the same limit; oversize is an explicit `AssetMetaError::DocumentTooLarge`
wrapped as invalid-data I/O for file APIs. A failed save leaves the prior file intact.

Discovery skips malformed or oversized sidecars with no paired compound directory. An oversized
`package.zmeta` paired with `package/` now fails the full source inventory with the original typed
oversize error, so members cannot be silently reinterpreted as independent assets. Safe tree-walk
errors retain priority over this deferred metadata error. The persisted-source identity check still
reopens the live sidecar through the bounded metadata loader, preserving its existing race check.
Duplicate metadata opens and the full projected inventory remain separate performance work.
Sparse oversized sidecar, paired-directory inventory, exact-limit, limit-plus-one growth, and
save-bound regressions are source candidates pending combined managed Cargo validation.
