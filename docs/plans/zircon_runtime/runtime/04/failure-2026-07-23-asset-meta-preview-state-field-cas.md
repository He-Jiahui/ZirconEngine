---
handoff_kind: failure
status: open
created_at: 2026-07-23
summary_slug: asset-meta-preview-state-field-cas
origin_plan: docs/plans/zircon_editor/editor/09-editor-asset-management.md
fixing_plan: docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
origin_child_dir: docs/plans/zircon_editor/editor/09
fixing_child_dir: docs/plans/zircon_runtime/runtime/04
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/asset/project/meta.rs
  - zircon_runtime/src/asset/project/meta_preview_state.rs
  - zircon_runtime/src/asset/project/meta_write_authority.rs
  - zircon_runtime/src/asset/project/manager/scan_and_import.rs
  - zircon_runtime/src/asset/project/manager/load_or_create_meta.rs
  - zircon_runtime/src/asset/project/manager/scan_and_import/targeted.rs
  - zircon_runtime/src/asset/tests/project/manager/targeted_import/meta_preconditions.rs
  - zircon_runtime/src/asset/migration/run.rs
  - zircon_runtime/src/asset/migration/run/meta_authority_tests.rs
  - zircon_runtime/crates/zr_resource/src/io/atomic_file
  - zircon_editor/src/ui/host/editor_asset_manager/manager/preview_refresh/request_preview_refresh.rs
tests:
  - cargo test -p zircon_runtime asset_meta_preview_state --locked
  - cargo test -p zircon_editor preview_refresh --locked
---

# Runtime04：AssetMeta preview_state 缺少字段级 CAS 更新合同

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/09-editor-asset-management.md`
- 来源执行切片：Editor09 asset catalog immutable generation / preview worker failure repair
- 修复责任计划：`docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md`
- 交接原因：`.zmeta` v7 及其原子写入由 Runtime04 持有；Editor09 只能消费该 authority，不能建立第二份 sidecar 或私有文件锁。

## 失败现象与复现证据

Editor09 preview worker 只需持久化 `PreviewState`，但 `AssetMetaDocument` 当前仅提供整文档
`load`/`save`。若 importer、watcher 或 migration 在 preview job 启动后更新同一 `.zmeta` 的
`source_digest`、`import_settings`、tags、entries 或 schema 字段，preview 若保存启动时的 clone
会整文件覆盖新字段；即使保存前重新 load 并合并 `preview_state`，外部写入仍可发生在 load 与
atomic replace 之间，缺少可证明的 compare-and-swap 边界。

2026-07-23 Editor09 独立复审将该 lost-update 窗口判为 Important。Editor09 已把 preview
decode/encode 移到 bounded worker，并在提交 generation 前校验 catalog revision、本资产 row identity、
source hash 与 meta path；剩余最低根因是 Runtime04 没有字段级 sidecar mutation authority。

## 最低共享层根因

`AssetMetaDocument::save` 是整文档无条件替换，Runtime asset owner 没有以
`{path, uuid, url, source_digest / document generation}` 为前置条件的字段级更新 API，也没有让
importer、migration 与 editor preview 共享同一路径写入序列。调用方无法在不复制 Runtime truth、
不覆盖其它字段的前提下只提交 `preview_state`。

## 架构修复验收

- Runtime04 提供 manager/meta authority 自有的 `preview_state` 字段级 CAS；更新必须重新读取当前
  `.zmeta`，校验 UUID、URL、source digest（或更强 document generation），仅修改 preview 字段。
- importer/watcher/migration 与 preview 字段更新走同一受管同路径写入序列；CAS 失败返回 typed stale
  结果，禁止覆盖当前文档。
- 并发测试用 barrier 在 preview read 与 commit 之间更新 settings/tags/entries/digest，最终文档保留
  外部字段；digest 变化时 preview update 必须 stale，digest 未变的独立字段更新也不得丢失。
- Editor09 worker 消费该 API 后，sidecar I/O 不持 editor live state lock 或全局 publish gate；最终只在
  短 gate 内验证并发布 generation row。

## 禁止临时方案

- 不得在 Editor09 新建 `.editor.meta`、第二份 preview sidecar、进程外 shadow truth 或调用点私有锁。
- 不得继续保存 preview job 启动时克隆的整份 `AssetMetaDocument`。
- 不得以“保存前再 load 一次”冒充原子 CAS，也不得弱化并发测试来隐藏 lost update。

## 修复结果与回传

Open state: `待 Runtime04 实现同路径字段级 CAS，并由 Editor09 复跑 preview worker 并发提交门；当前不声明 sidecar persistence 已通过。`

### 2026-09-09 source audit and migration authority repair

The Runtime field CAS already exists in `meta_preview_state.rs`: it acquires
the shared physical-path authority, rereads the current document, returns typed
UUID/URL/source-digest stale results, and changes only `preview_state`.
Managed Windows job `5f37c26c5bef47629a7ddfabdcd68470` executed the filter
`asset_meta_preview_state` with `zircon_runtime`, no default features, static
linking and locked dependencies: 3 passed, 0 failed, 0 ignored, exit 0.
The source input is
`E:/cargo-targets/zircon-engine/cache/build-benchmarks/runtime04-artifact-3322-20260909`,
manifest `03e62d73910441d6bfecf66677d1ef1c0d6ccf5269ecae3098936a8f63729520`;
`results/runtime04-meta-preview-3322-r1.json` and its sibling log bind the
actual command, managed receipt and test output. The CAS implementation,
meta schema, authority and the file containing those three tests matched that
immutable input.

The remaining Runtime gap was concrete: migration bypassed the meta authority
during sidecar preparation, recovery and durable commit. Acquiring a lock only
at commit would still publish candidate bytes read before a concurrent update.
`migration/run.rs` now acquires all inventory transaction paths before recovery
and preflight and retains them until commit finishes. After recovery it releases
the old guard set, rebuilds the inventory and acquires the complete new set
before reading candidate documents. Error and dry-run exits release the guards.
No second path-lock implementation or sidecar truth was introduced.

Snapshot `3325` freezes `migration/run.rs` at
`f7266acd91defe85fcb39cd024924a8d5fcb8e50bdf493b7ce7e13abdd3b69b9`
and `migration/run/meta_authority_tests.rs` at
`57681bab94253d70e342a85560c838590051adaa00b2b393d0607d6458c93884`.
The new concurrent regressions cover a writer publishing independent fields
before migration preflight and preview CAS waiting until migration publishes
the converted v7 sidecar. The commit hook pauses the real migration path;
it does not substitute a test implementation for preparation or transaction I/O.
Scoped rustfmt checking and `git diff --check` passed.

Managed Windows job `c0d8c008ec7d4412b632cbbf29009778` then executed all five
`asset_meta_preview_state` tests, including both new migration concurrency
regressions: 5 passed, 0 failed, 0 ignored, exit 0. Its immutable input is
`E:/cargo-targets/zircon-engine/cache/build-benchmarks/runtime04-meta-cas-3325-20260909`,
manifest `9d5e59424e1729c118aa250bd929eea1f7b918871bc5eb54504e4ad61c7243a3`.
`results/runtime04-meta-cas-3325-r1.json` and its sibling log bind the locked,
no-default-feature lib-test command and managed receipt to that input.

The broader migration job `75e6b363944e4617b802b9064e274677` on the same
input executed 63 passing, 8 failing and 1 ignored tests. Its original evidence
is retained in `results/runtime04-migration-3325-r1.json` and the sibling log.
The eight stale fixture-contract failures have their own local lifecycle:
`failure-2026-09-09-migration-current-contract-fixtures.md`. Snapshot `3329`
repairs those tests without replacing the production GUID authority; the new
public migration receipt remained required at that point.

The wider migration regression subsequently passed as managed Windows job
`9aa5c29f3f6640a1808d55c69973254f`: 71 passed, 0 failed, 1 ignored, exit 0,
using the locked static/no-default `asset::tests::migration` filter. Input
`runtime04-migration-fixtures-3329-20260909` has manifest
`3d03d4e4fdb130c29e772a3fe2d08d29459a5e3737f0a24fa41ef932b42fe644`;
`results/runtime04-migration-3329-r2.json` and its sibling log retain the exact
command, receipt and executed test names. The migration authority source and
its concurrency tests still match `3325`. This passes the ordinary migration
recovery/regression batch; the ignored scale test and later targeted-import
continuation remain outside that result.

Editor09's `complete_preview_refresh_job` still only updates its in-memory
record after loading meta; it does not consume `compare_and_set_preview_state`.
Its existing handoff
`docs/plans/zircon_editor/editor/09/failure-2026-07-17-editor-asset-catalog-full-rebuild-and-preview-lock.md`
remains the consumer owner. Editor live-lock/publish-gate acceptance, independent
C0/I0/M0 review, failure return and closeout remain pending. No commit SHA or
WeCom delivery is claimed for this open lifecycle.

### 2026-09-09 targeted importer continuation

The later targeted-import audit found a second concrete preparation window:
single-source and batch commits acquired the shared meta authority but could
still overwrite a CAS completed after their earlier meta read. Source snapshot
`3331` now validates the exact original meta projection or absence under those
guards before any durable file transaction. It retains existing importer
input-digest changes and does not hold a path guard across importer execution.
The linked `failure-2026-07-18-project-source-index-targeted-import.md` records
the source hashes and five new regressions. They remain dynamically unverified;
the earlier five-test CAS pass is not reused as evidence for this new importer
path. Editor09 consumption and the complete lifecycle acceptance remain open.

The first importer continuation input (`runtime04-targeted-meta-3331-20260909`,
manifest `ad8d4b698f545c24ee19389ed6a463cad1916b1afe0e4ca2a9bd1e95540447d2`)
failed compilation in job `ee56cd17f18f41b1befda3ef0282ed84`, with zero tests.
Its inherited compiler input lacked the build-identity producers already used
by the current targeted source. The three missing symbols were
`canonical_import_input_digest`, `build_identity_for_import`, and
`AssetImportContext::with_build_identity`. This is an incomplete dependency
snapshot, not a behavioral RED for the new metadata guards. The original log
and JSON receipt remain under `results/runtime04-targeted-meta-3331-r1`;
the next targeted attempt must include matching importer, build-script and
Cargo inputs. No dependency API was reverted to make the old input compile.
