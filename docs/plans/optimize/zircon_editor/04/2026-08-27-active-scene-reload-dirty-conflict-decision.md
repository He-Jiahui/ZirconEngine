# Active-scene dirty reload decision flow

## Problem

The generation-fenced active-scene reload correctly retained dirty local history, but its first
slice stopped at a status line. An operator could not resolve the conflict through the Editor's
notification authority, and a repeated watcher event could reopen the same unresolved condition.
The missing workflow was part of the Editor MVP: external source changes must never silently erase
authoring work, while Save, explicit Discard, and Keep Editing must remain bounded and deterministic.

## Reference review

The local Unreal source was reviewed before changing the coordinator. In
`dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/PackageTools.cpp`, reload separates dirty
packages and only clears their dirty state after an explicit revert decision. In
`dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/FileHelpers.cpp`, save prompting has one
authority, typed terminal results, and cancellation aborts the transition. Zircon follows the same
structural rules: no implicit dirty loss, one decision owner, and world replacement only after the
operator intent and terminal identity checks agree.

## Architecture

`ActiveSceneReloadConflict` now binds the lifecycle-owned `ActiveSceneDocumentIdentity` to the
Runtime `ProjectAssetGenerationToken` and one of three states: `AwaitingDecision`,
`DiscardRequested`, or `Cancelled`. The context-owned `DecisionNotificationCenter` publishes one
three-option Decision with a monotonic notification sequence. The retained host tracks the exact
ticket; a pending ticket is not guessed, a resolved ticket is consumed once, and an evicted ticket
causes an explicit republication. Keep Editing suppresses the exact identity/generation, while a
new Runtime generation re-enters conflict detection. Display-only scene URIs are UTF-8 safely
bounded to the notification payload limit.

The first implementation incorrectly routed Save through `EditorDirtySaveCoordinator` using the
lifecycle document id. Code review proved that coordinator owns registered toolkit documents and
`DirtyRegistry`, while scene dirtiness belongs to the Global `EditorTransactionEngine` history and
the authoritative save route is `MenuAction::SaveProject -> EditorManager::save_active_scene ->
mark_saved_if_unchanged`. The provisional pending/save states and adapter were therefore removed.
Conflict Save now invokes that one project-scene authority, verifies that the exact scene identity
is still active and Global history is clean, then queues one latest-generation reload. Save failure
or a still-dirty history restores an explicit Decision rather than reloading.

The toolkit coordinator still needed a separate competition repair: it now acquires one typed
`DirtyDocumentSaveOwner` for Save All or ClosePrompt before job admission and rejects non-owner
completion polling. Save All retains one queued request behind a close-prompt batch. Project and
native-window close query the same owner before teardown; a direct project-close commit has the same
guard before model-import cancellation, palette mutation, Play shutdown, or manager close. This
does not create a second serializer or dirty registry.

Discard is a typed `PreparedActiveSceneReloadDirtyPolicy`. It skips the dirty admission check but
does not mutate history during job preparation. The existing lifecycle coordinator rechecks the
complete activation identity, and `commit_if_project_generation` retains the Runtime generation
fence through `reload_active_scene_world`. Only that terminal closure clears history and installs
the prepared world. Identity or generation supersession, load failure, or install failure retains
the local dirty state and reopens the decision when the same scene remains active.

## Evidence and status

- Target conflict contract first moved red-to-green from missing conflict module to 12/12. The
  ownership regression was then added red-to-green, and the current Editor10 static suite passes
  15/15 (focused generation/competition file 13/13); scoped `rustfmt --check` and `git diff --check` pass.
- Rust behavior coverage now includes wrong-owner polling against a real completed batch, Save All
  queue/acquire after a close-prompt save, and Project Close remaining in Project mode until that
  save terminalizes.
  These tests are authored but not counted as executed because the editor crate did not compile.
- The current full Editor09 pattern has three unrelated shared-tree failures: the retired
  `core/editor_plugin.rs` path, a changed dirty-facade export set, and a missing crate-root watcher
  poll re-export. They are not counted as this slice's result.
- E-drive `cargo check -p zircon_editor --lib --offline` reached a terminal result in 161.3 seconds,
  but `zircon_runtime` failed first with 61 errors and 123 warnings. The editor owner did not compile,
  so this is neither an owned pass nor an owned failure.
- A later isolated E-drive check remained in `zircon_runtime` and timed out after 364.2 seconds
  without diagnostics; its exact Cargo/rustc process tree was retired and no editor source compiled.
- The first follow-up review confirmed the original two P1 competition blockers were removed and
  withdrew the earlier `schedule(false)` interpretation after reading the adapter. It then found a
  static-test scope error and missing real completion coverage; both are corrected. The final
  independent source review returned `READY` with no P1/P2 findings after rechecking the canonical
  SaveProject route, exact identity plus clean-history gate, owner-first completion polling, queued
  Save All continuation, and close teardown barriers. Rust behavior execution and F0/F4 product
  traces remain pending. The parent
  performance failure stays open and no milestone commit or WeCom acceptance message is permitted.

## Remaining work

Managed behavior coverage must exercise all three operator choices, project-scene save failure, a
Decision receipt eviction, identity replacement, same-project generation supersession, admission
backoff, and project close during an owned save. Product evidence must still report event-to-commit
wall time, UI blocked time, scene-load and terminal-install time, RSS, and power. Runtime-extension
application, level allocation, and authoring seed construction remain on the UI thread until their
thread-safety boundary is proven and profiled.
