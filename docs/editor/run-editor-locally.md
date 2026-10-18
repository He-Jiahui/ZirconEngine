# Run a staged editor locally (Windows)

Use Rust 1.94.1 and an explicit Python 3.11+ interpreter. This independent preview
is authorized for local checks; it does not grant Jenkins or milestone acceptance.
The build wrapper rejects output paths outside physical drive-root
`D/E/F:\cargo-targets\zircon-local` and preserves the BuildSet manifest checks.

```powershell
$stamp = Get-Date -Format 'yyyyMMdd-HHmmss'
& tools/build/run-editor-preview.ps1 `
  -PythonInterpreter 'E:\Tools\Python311\python.exe' `
  -OutputDirectory "E:\cargo-targets\zircon-local\editor-preview-$stamp" `
  -TargetDir 'E:\cargo-targets\zircon-local\windows\editor-development\target'
```

The output parent must exist and the destination must be fresh. The command builds
and stages the EXE, DLL, assets and matching sidecar, then launches a new
RenderableEmpty project with `ZIRCON_EDITOR_EXIT_AFTER_FIRST_FRAME=1` and
`ZIRCON_EDITOR_CAPTURE_FIRST_FRAME_PNG`. It keeps logs, project, capture and a
`startup-smoke-receipt.json` in the reported evidence directory. A timeout, nonzero
exit, missing/blank image, missing GPU diagnostic or software fallback fails the
smoke. The build receipt records the EXE/DLL hashes and smoke result. A passing
first-frame check proves startup, not editing behavior, visual parity or PIE speed.

To keep working, launch the staged `zircon_editor.exe --project <project-path>`
without the exit-after-first-frame environment variable. Keep its matching DLL
and `.manifest.json` sidecar together. Missing or mismatched manifests remain
startup errors; inspect `first-frame.stderr.log` for preflight recovery guidance.

`WGPU_BACKEND=dx12` or `vulkan` restricts adapters explicitly. With no override,
Windows runtime selection prefers DX12, then Vulkan. The runtime-shared chrome
presenter uses that runtime's device; standalone/degraded presenter evidence must
be assessed separately. Repeat the preview with each supported backend and at
100%/150% DPI before claiming product acceptance.
