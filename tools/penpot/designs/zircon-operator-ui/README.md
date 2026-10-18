# Zircon Operator UI design inputs

This directory reserves the input-of-record path from [plan 14](../../../../docs/plans/zircon_editor/editor_ui/14-penpot-operator-ui.md).
No actual Penpot Foundations, Components or Screens file has been authored or
exported for this batch. The compiler's `src/fixture.mjs` and golden are explicitly
synthetic engineering tests; they are not product design or visual evidence.

In the plugin's **Annotate** tab, load the reviewed
`zircon_editor/assets/ui/operator/contract/roles.contract.toml`. Select each
semantic layer and apply JSON such as:

```json
{"role":"project.browser.open","command":"file.project.open","text":"i18n:command.file.project.open.label"}
```

Annotate the screen root, required slots and commands. Use active local `editor.*`
foundation tokens and versioned local `zop/<Component>/md` components. ZOP Export
works on fresh boards and returns a ZIP containing the actual file ID/revision,
normalized shapes, resolved DTCG tokens, actual board PNG and SHA-256 manifest.
An edit during export invalidates the capture. Connected-library revision export,
SVG icons, repeaters, state expressions and the full component/state kit remain
later work; this version rejects unsupported input instead of approximating it.

Token bindings must match the DTCG type: fill/stroke colors use `color`, font
families use `fontFamily`, font sizes use `number` or a `px` dimension, and font
weights use `number`. Inherited group types are supported. Type mismatches fail
compilation with the board/node path and token name before any output is applied.

Land a reviewed, extracted export at `bundle/**`, replacing its placeholder note.
Keep its manifest and hashes intact. Build/verify/apply with the editor closed:

```powershell
$out = 'E:\cargo-targets\zircon-local\penpot\zop-20261017\out'
$bundle = Join-Path $PWD 'tools\penpot\designs\zircon-operator-ui\bundle'
pnpm --dir tools/penpot --filter zircon-zop-compiler compile --bundle $bundle --output $out
pnpm --dir tools/penpot --filter zircon-zop-compiler apply --from $out
pnpm --dir tools/penpot --filter zircon-zop-compiler verify --bundle $bundle
```

Package scripts run with the compiler package as their current directory; pass an
absolute `--bundle` path when invoking through pnpm. Artifact writes require real
Windows `D/E/F:\cargo-targets` storage. Apply refuses unowned/locally modified
destinations, preserves the contract, and retires only unchanged files owned by
the preceding lock. A compiler/export test pass is not native startup acceptance.

For no-output Linux contract tests:

```sh
node --test tools/penpot/apps/zircon-zop-compiler/src/compiler.test.mjs tools/penpot/apps/zircon-zop-compiler/src/export.test.mjs
```

Full native-schema parity, plugin type checking/build and Penpot host tests use
`tools/penpot/tools/scripts/run-validation.ps1 -ArtifactRoot E:\cargo-targets\zircon-local\penpot\zop-<unique>`.
