# Penpot tools

ZirconEngine owns the ZUI plugin in `apps/zircon-zui-plugin` and its integration
scripts here. The upstream dependency is a source snapshot in
`../../third_party/penpot`, pinned by `UPSTREAM.json`; it has no nested Git
repository. Upstream licenses and package lockfiles are retained. npm packages
remain exact locked dependencies, rather than checked-in `node_modules`.

Use the Node version in `.nvmrc` and pnpm from `packageManager`. From the
ZirconEngine repository root:

```powershell
$artifacts = 'E:\cargo-targets\zircon-local\penpot\review-20261004'
& tools/penpot/tools/scripts/run-validation.ps1 -ArtifactRoot $artifacts
```

The directory must be fresh and physically under
`D/E/F:\cargo-targets\zircon-local`; junctions and symlinks are rejected. The
runner copies versioned inputs there, installs with `--frozen-lockfile`, then
tests, typechecks, lints and builds. `-TestsOnly`, `-LintOnly` and `-BuildOnly`
select narrower gates. `-Offline -CacheSource <existing-pnpm-store>` copies an
existing store into the approved output directory before using it. Source
directories receive no generated build outputs or dependency caches.

`inputs.json` records staged input hashes; `validation.json` records command
exit codes and logs. The distributable plugin is
`<artifacts>/zircon-zui-plugin/dist`, including the independently built and
hash-checked `assets/plugin.js`. To preview a successful build:

```powershell
pnpm --dir "$artifacts/workspace/tools/penpot" exec vite preview --outDir "$artifacts/zircon-zui-plugin/dist" --host 127.0.0.1 --port 4213
```

Add `http://127.0.0.1:4213/manifest.json` to Penpot. Headless bridge commands
also run from the staged workspace; their output paths must stay in the
approved artifact directory.

The separate workbench entry preserves browser capture modes:

```powershell
& tools/penpot/tools/scripts/run-managed-zui-plugin-validation.ps1 -CaptureMode render-only
```

It retains artifacts without starting the retired local coordinator. Capture
uses the official frontend and repository RPC mocks; render-only evidence
remains pending, and persistence and engine visual acceptance remain separate.

Layout catalog `.zui` inputs and mirrors live in `docs/ui/zui`; catalog JSON,
images and review records remain in `docs/_data/layout`. Catalog paths remain
logical relative paths, resolved by `zui-layout-paths.ts`. Existing historical
evidence is preserved, and changed source/program hashes require new evidence.

To update Penpot, import a reviewed upstream commit into `third_party/penpot`,
refresh its provenance manifest, then update this workspace's lockfile and
validate. Product changes belong here, rather than in `dev/penpot`.
