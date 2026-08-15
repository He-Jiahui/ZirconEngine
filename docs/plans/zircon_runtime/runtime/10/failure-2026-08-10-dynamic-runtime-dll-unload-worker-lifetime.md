---
handoff_kind: failure
status: open
created_at: 2026-08-10
summary_slug: dynamic-runtime-dll-unload-worker-lifetime
origin_plan: docs/plans/mvp/06-f5-acceptance-wave.md
fixing_plan: docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md
origin_child_dir: docs/plans/mvp/06
fixing_child_dir: docs/plans/zircon_runtime/runtime/10
plan_link_mode: child_record_only
related_code:
  - zircon_app/src/entry/runtime_library/loaded_runtime.rs
  - zircon_app/src/entry/runtime_library/runtime_session.rs
  - zircon_runtime/src/dynamic_api/session/ffi.rs
  - zircon_runtime/src/diagnostic_log/sink.rs
  - zircon_runtime/src/diagnostic_log/sink/worker.rs
tests:
  - .\\target\\profiling\\zircon_editor.exe --operation view.hierarchy.open --headless
  - repeated dynamic runtime create/destroy followed by actual library unload
  - fresh F5 staged product sequence and evidence validation
---

# Runtime10: dynamic runtime DLL unloads before its diagnostic worker exits

## 来源执行者

- 来源计划：`docs/plans/mvp/06-f5-acceptance-wave.md`
- 来源执行切片：F5 product acceptance gate, headless editor operation smoke.
- 修复责任计划：`docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md`
- 交接原因：Runtime10 owns the `LoadedRuntime`/`RuntimeSession` cdylib boundary, its
  session teardown contract, and the dynamic runtime's process-local diagnostic worker lifetime.

## 失败现象与复现证据

The existing Windows profiling product can capture a valid first editor frame, but the rapid
headless operation path is not process-stable. Repeating:

```powershell
.\target\profiling\zircon_editor.exe --operation view.hierarchy.open --headless
```

produced successful operation JSON on seven of eight attempts and exited with `0xc0000005` on one
attempt. Windows Application Error event 1000 reports `zircon_runtime.dll_unloaded` as the
fault module, with the faulting address at `zircon_runtime.dll + 0x2138eeb`. The executable and
DLL were pre-existing profiling artifacts rather than a fresh managed current-source build, so
this is product-fault evidence, not current-source acceptance evidence.

The product also produced a non-empty 1672x941 first-frame PNG for the same project. That proves
the baseline editor surface launched; it does not make the unstable headless operation acceptable
for F5.

## 最低共享层根因

The reproduced source state let `RuntimeSession::drop` destroy the session and then let
`LoadedRuntime` drop its `libloading::Library` while the dynamic runtime's process-local
`OnceLock` diagnostic sink still owned a named `zircon-diagnostic-log` worker. The dynamic session
FFI initialized that worker without tying it to the final session destroy path.

The current forward repair replaces that permanent sink slot with a generation controller and a
dynamic-session lease. The final dynamic session first quiesces its watchers, then unpublishes,
stops, and joins the DLL worker before its destroy call returns; a later dynamic session creates a
fresh sink generation. Worker liveness is now distinct from output durability, so a joined worker
with a failed output does not cause an unsafe unload deferral.

The host no longer hides a failed destroy by forgetting the wake registration or
`libloading::Library`. A non-OK destroy is an unrecoverable safety failure because neither the
runtime worker nor a copied host callback can be proven stopped; the host records the status and
terminates before normal Rust drop could unload the DLL.

The session registry's action/wake quiescence protects session callbacks, but it is not a
library-process shutdown protocol. The missing DLL-worker quiescence is therefore the narrowest
shared boundary consistent with the unloaded-module fault. No DLL PDB was available for a source
line mapping, so the exact instruction is not claimed as symbolized proof.

## 架构修复验收

- Define one Runtime10-owned library shutdown/quiescence protocol that runs before the final
  `Library` drop, joins every DLL-owned worker, and retains the library until those joins finish.
- Preserve multi-session semantics: destroying one session cannot disable diagnostics or unload
  the library while another session is live; a later valid session must not depend on stale
  process-global state.
- Exercise the real dynamic-loader path through repeated create/destroy and final unload,
  including a sink-enabled worker, and assert no worker remains executable from the unloaded DLL.
- Run a freshly built, source-bound Windows product at least twenty times through
  `view.hierarchy.open --headless`; every attempt must exit zero and return the successful typed
  operation result.
- Rerun the full F5 staged create/render/edit/reopen evidence sequence only after the lower-layer
  lifetime regression is green. The old profiling artifact and its screenshot cannot be reused as
  acceptance evidence.

## 禁止临时方案

- Do not `mem::forget` or otherwise intentionally retain `libloading::Library` to hide the race.
- Do not add sleeps, retries, process-exit-only cleanup, test-only teardown, or disable the
  diagnostic sink to make the headless operation appear stable.
- Do not unload per session without a final-library-owner protocol, and do not weaken F5's
  repeated product-process or evidence requirements.

## 修复结果与回传

Open state: `Runtime10 source forward repair and focused regressions recorded / managed validation
pending / F5 product acceptance remains pending`. Scoped Rust 2021 formatting and diff checks pass;
the new regression tests and a fresh managed Windows build have not yet run, and no passing repeated
product claim is made by this record.

The post-lease construction-failure branch now explicitly releases the dynamic diagnostic lease.
If its worker cannot be joined, it terminates before returning the construction error, so a failed
`create_session` cannot hand an unloadable DLL back to the caller with a live runtime worker.

Independent source review on 2026-08-10 traced every current dynamic-session exit path. The
registry first quiesces actions and wake callbacks, then invokes
`RuntimeDynamicSession::shutdown_before_library_unload`; that stops project watchers before the
final diagnostic lease. A failed lease shutdown remains a non-OK destroy status while the host
still owns `LoadedRuntime`, and the host takes its abort path rather than reaching normal library
drop. The session destructor reruns the same shutdown hook only after the registry path has
captured that status, so it does not turn the failure into success. This is static source review
only: the dynamic-loader regression and fresh product repetitions remain required before this
handoff can move to fixed.
