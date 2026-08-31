---
handoff_kind: failure
status: open
created_at: 2026-08-12
summary_slug: native-plugin-development-watcher-job-ownership
origin_plan: docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
fixing_plan: docs/plans/zircon_editor/editor/12-plugin-management.md
origin_child_dir: docs/plans/zircon_editor/editor/14
fixing_child_dir: docs/plans/zircon_editor/editor/12
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/ui/retained_host/app/module_plugin_actions/live_host/development_watch.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_actions/live_host.rs
  - zircon_editor/src/core/jobs/tests/thread_ownership_contract.rs
tests:
  - cargo test -p zircon_editor --lib development_watch --locked --jobs 1 -- --test-threads=1
  - cargo test -p zircon_editor --lib editor_production_sources_do_not_create_bare_threads --locked --jobs 1 -- --test-threads=1
---

# Editor12: native plugin development watcher must use the canonical job owner

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md`
- 来源执行切片：M2 complete production bare-thread ownership audit
- 修复责任计划：`docs/plans/zircon_editor/editor/12-plugin-management.md`
- 交接原因：native plugin artifact watching, debounce, reload ordering, and its diagnostic lifecycle are plugin
  management behavior. Editor14 owns the global no-bare-thread gate but cannot replace a plugin lifecycle protocol
  with a second generic worker implementation.

## 失败现象与复现证据

`zircon_editor/src/ui/retained_host/app/module_plugin_actions/live_host/development_watch.rs:59-135` stores a
private `JoinHandle`, creates `mpsc::sync_channel(1)`, starts `thread::Builder::spawn`, and joins that worker from
`Drop`. The worker waits for debounce events and invokes `NativePluginLiveHost::hot_reload_editor_plugin` directly.

This is a production direct-thread owner, so it violates the Editor14 M2 hard-cut criterion independently of the
existing `notify` backend. The crate-wide ownership guard is the upward reproduction; its direct-thread result must
become clean without a source whitelist or an alias.

## 最低共享层根因

The plugin development watcher treats notification coalescing, debounce delay, reload execution, cancellation, and
terminal join as one private retained-host thread protocol. The callback has no canonical EditorJobSystem admission,
ticket, cancellation, shutdown, or diagnostic ownership, and `Drop` can wait for the worker on a UI-owned path.

## 架构修复验收

- Editor12 injects the existing `EditorJobSystem` at the plugin-host composition boundary and represents the
  development watcher reload lifecycle as a typed, cancellable job/ticket owned by that system.
- File notifications stay bounded and coalesced; debounce, reload ordering, host disappearance, cancellation,
  fault diagnostics, and shutdown have explicit ticket semantics. No retained-host `JoinHandle`, `mpsc` worker
  channel, direct `std::thread`, or UI-thread join remains.
- Exact-artifact filtering and reload diagnostics keep their current behavior. The focused development-watch test,
  the Editor14 production thread-ownership gate, and the appropriate plugin-management gate pass from an immutable
  current-source manifest and receive independent review.

## 禁止临时方案

- Do not whitelist this path in the global ownership guard, rename or alias the thread API, or retain a wrapper
  around a private worker.
- Do not move reload work into the `notify` callback, block the UI thread for debounce/join, or duplicate the
  Runtime11 blocking-I/O stream owner.
- Do not preserve a compatibility facade that keeps both the old watcher thread and the canonical job lifecycle.

## 2026-08-29 current-source architecture recheck

- Current `development_watch.rs` still owns a `sync_channel(1)`, an `AtomicBool`, a named
  `std::thread::JoinHandle`, a 350 ms blocking debounce loop, and a synchronous `Drop::join`.
  The only concurrent drift in that file is the already-applied native-plugin module path hard cut;
  this repair preserves that current blob instead of replaying the historical file.
- The current `EditorJobSystem` already owns bounded admission, cancellation, mutex groups, ticket
  completion, shutdown, progress, and event diagnostics. Adding another watcher worker would leave
  two scheduling authorities and keep UI teardown coupled to a foreign thread.
- Unreal's `IDirectoryWatcher` exposes change callbacks plus an editor-driven `Tick`; `HotReload.cpp`
  records changed binaries in the watcher callback and performs the reload decision from the hot
  reload ticker. The relevant architectural rule is that the watcher publishes change facts while
  the editor scheduler owns execution and teardown.
- Zircon will therefore keep exactly one latest-change timestamp and at most one canonical job
  ticket per watched plugin. Retained Host tick consumes a due timestamp after the debounce window,
  submits one background `EditorJob` in the global native-plugin reload mutex group, and polls its
  ticket without blocking. Changes arriving while a job is pending or running overwrite the single
  timestamp and schedule at most one later reload.
- Watch removal drops the OS watcher and cancels the current ticket token without waiting. There is
  no retained-host join, private channel, private executor, delayed worker sleep, or compatibility
  path. The weak native-host handle remains the terminal host-disappearance guard.

## 修复结果与回传

Open state: `source hard cut complete / static contracts green / managed Rust and product evidence pending`.
The private watcher worker has been removed from current source. The handoff remains `open` until the immutable
managed Rust gate and independent review are recorded; no `fixed-*` return is claimed yet.

## 产出记录与时间

| 时间 | 状态 | 完成项目 | 证据 |
|---|---|---|---|
| 2026-08-12 | `open / Editor14-forward-handoff-recorded` | Complete production-source audit identified the native plugin development watcher as a raw-worker owner after the profile writer repair. | `development_watch.rs:59-135`; this record is the canonical Editor12 fixing artifact. No source mutation or validation is claimed. |
| 2026-08-29 | `source_complete / static_green / managed_validation_pending` | Hard-cut `sync_channel(1)`, `AtomicBool`, private `JoinHandle`, blocking debounce worker, and `Drop::join`. Exact artifact events now overwrite one timestamp and signal the Host wake edge; retained tick merges the 350 ms due time with the project heartbeat, submits one background Compile ticket through `EditorJobSystem`, polls completion, and cancels without waiting. Native reload jobs share one global mutex group and keep the weak-host terminal guard. | Current-source owner scan: `EditorJobSystem=1`, `JobTicket<String>=1`, `changed_at` slot=1, ticket slot=1; production `std::thread`/`JoinHandle`/`sync_channel`/`RecvTimeoutError`/`.join()` all zero. Constructor inventory 2/2 and backend impl inventory 3/3 migrated; scoped rustfmt and diff whitespace pass. Managed Cargo/product and independent review remain pending, so this Failure stays open. |

## 2026-08-27 module-root ownership anchor repair

The canonical Rust module root is the existing sibling file
`module_plugin_actions/live_host.rs`, which mounts the `live_host/` child modules; no
`live_host/mod.rs` owner exists in repository history. The structured anchor now names
that current root. Development-watch implementation bytes were not modified and no
managed Editor validation was added, so the job-ownership failure remains open.
