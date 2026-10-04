use crate::core::framework::project::{ExportProfile, ExportTargetPlatform};

use super::super::ExportGeneratedFile;
use super::{
    html_escape, javascript_string_escape, json_string_escape, native_library_stem,
    runtime_library_file,
};

pub(super) fn browser_host_files(profile: &ExportProfile) -> Vec<ExportGeneratedFile> {
    let (host_name, script_name, script_contents, readme_title) = match profile.target_platform {
        ExportTargetPlatform::WebGpu => (
            "webgpu",
            "src/zircon_webgpu_host.js",
            webgpu_host_script_template(profile),
            "WebGPU browser host",
        ),
        ExportTargetPlatform::Wasm => (
            "wasm",
            "src/zircon_wasm_host.js",
            wasm_host_script_template(profile),
            "WASM browser host",
        ),
        _ => return Vec::new(),
    };
    vec![
        runtime_library_file(profile, readme_title),
        ExportGeneratedFile {
            path: format!("platform/{host_name}/index.html"),
            purpose: format!("{readme_title} HTML shell"),
            contents: browser_index_template(profile, script_name),
        },
        ExportGeneratedFile {
            path: format!("platform/{host_name}/{script_name}"),
            purpose: format!("{readme_title} JavaScript launcher"),
            contents: script_contents,
        },
        ExportGeneratedFile {
            path: format!("platform/{host_name}/package.json"),
            purpose: format!("{readme_title} package manifest"),
            contents: browser_package_json_template(profile, host_name),
        },
        ExportGeneratedFile {
            path: format!("platform/{host_name}/vite.config.mjs"),
            purpose: format!("{readme_title} dev and release server config"),
            contents: browser_vite_config_template(host_name),
        },
        ExportGeneratedFile {
            path: format!("platform/{host_name}/public/zircon-export.manifest.json"),
            purpose: format!("{readme_title} fetch manifest"),
            contents: browser_fetch_manifest_template(profile, host_name),
        },
        ExportGeneratedFile {
            path: format!("platform/{host_name}/public/_headers"),
            purpose: format!("{readme_title} CDN cache headers"),
            contents: browser_cdn_headers_template(),
        },
        ExportGeneratedFile {
            path: format!("platform/{host_name}/public/zircon-export.cdn-manifest.json"),
            purpose: format!("{readme_title} CDN deployment manifest"),
            contents: browser_cdn_manifest_template(profile, host_name),
        },
        ExportGeneratedFile {
            path: format!("platform/{host_name}/package-export.mjs"),
            purpose: format!("{readme_title} release packaging script"),
            contents: browser_package_script_template(host_name),
        },
        ExportGeneratedFile {
            path: format!("platform/{host_name}/deploy-cdn.mjs"),
            purpose: format!("{readme_title} CDN deployment contract"),
            contents: browser_deploy_cdn_script_template(host_name),
        },
        ExportGeneratedFile {
            path: format!("platform/{host_name}/README.md"),
            purpose: format!("{readme_title} release packaging instructions"),
            contents: browser_readme_template(profile, readme_title),
        },
    ]
}

fn browser_package_json_template(profile: &ExportProfile, host_name: &str) -> String {
    format!(
        "{{\n  \"name\": \"zircon-export-{}-{}\",\n  \"version\": \"0.1.0\",\n  \"private\": true,\n  \"type\": \"module\",\n  \"scripts\": {{\n    \"dev\": \"vite --host 127.0.0.1\",\n    \"build\": \"vite build\",\n    \"preview\": \"vite preview --host 127.0.0.1\",\n    \"package:export\": \"node package-export.mjs\",\n    \"deploy:cdn\": \"node deploy-cdn.mjs\"\n  }},\n  \"devDependencies\": {{\n    \"@vitejs/plugin-basic-ssl\": \"latest\",\n    \"vite\": \"latest\"\n  }}\n}}\n",
        json_string_escape(host_name),
        json_string_escape(&native_library_stem(&profile.output_name))
    )
}

fn browser_vite_config_template(host_name: &str) -> String {
    format!(
        "import {{ defineConfig }} from 'vite';\n\nexport default defineConfig({{\n  base: './',\n  publicDir: 'public',\n  build: {{\n    outDir: 'dist/{}',\n    emptyOutDir: true,\n    target: 'es2022',\n    assetsInlineLimit: 0\n  }},\n  server: {{\n    headers: {{\n      'Cross-Origin-Opener-Policy': 'same-origin',\n      'Cross-Origin-Embedder-Policy': 'require-corp'\n    }}\n  }}\n}});\n",
        javascript_string_escape(host_name)
    )
}

fn browser_fetch_manifest_template(profile: &ExportProfile, host_name: &str) -> String {
    format!(
        "{{\n  \"profile\": \"{}\",\n  \"target\": \"{}\",\n  \"resourceStrategy\": \"browser_fetch\",\n  \"projectManifest\": \"../../assets/zircon-project.toml\",\n  \"allowedAssetRoot\": \"./assets/\",\n  \"wasmModule\": \"./zircon_export_{}.wasm\"\n}}\n",
        json_string_escape(&profile.name),
        json_string_escape(host_name),
        json_string_escape(&native_library_stem(&profile.output_name))
    )
}

fn browser_cdn_headers_template() -> String {
    "/*\n  Cross-Origin-Opener-Policy: same-origin\n  Cross-Origin-Embedder-Policy: require-corp\n  Cache-Control: public, max-age=300\n/assets/*\n  Cache-Control: public, max-age=31536000, immutable\n/*.wasm\n  Cache-Control: public, max-age=31536000, immutable\n  Content-Type: application/wasm\n"
        .to_string()
}

fn browser_cdn_manifest_template(profile: &ExportProfile, host_name: &str) -> String {
    format!(
        "{{\n  \"profile\": \"{}\",\n  \"target\": \"{}\",\n  \"baseUrl\": \"${{ZR_CDN_BASE_URL}}\",\n  \"immutableAssetPath\": \"assets/\",\n  \"compression\": [\"br\", \"gzip\"],\n  \"assetIntegrity\": \"sha256 manifest generated by CI before publish\"\n}}\n",
        json_string_escape(&profile.name),
        json_string_escape(host_name)
    )
}

fn browser_package_script_template(host_name: &str) -> String {
    format!(
        "import {{ mkdir, copyFile }} from 'node:fs/promises';\nimport {{ dirname, join }} from 'node:path';\n\nconst output = join('dist', '{}');\nawait mkdir(join(output, 'assets'), {{ recursive: true }});\nawait copyFile('../../assets/zircon-project.toml', join(output, 'assets', 'zircon-project.toml'));\nconsole.log(`Browser export assets staged in ${{output}}`);\n",
        javascript_string_escape(host_name)
    )
}

fn browser_deploy_cdn_script_template(host_name: &str) -> String {
    format!(
        "import {{ createHash }} from 'node:crypto';\nimport {{ brotliCompress, gzip }} from 'node:zlib';\nimport {{ promisify }} from 'node:util';\nimport {{ access, readdir, readFile, writeFile }} from 'node:fs/promises';\nimport {{ join, relative }} from 'node:path';\nimport {{ execFile }} from 'node:child_process';\n\nconst brotli = promisify(brotliCompress);\nconst gzipAsync = promisify(gzip);\nconst execFileAsync = promisify(execFile);\nconst baseUrl = process.env.ZR_CDN_BASE_URL;\nconst uploadCommand = process.env.ZR_CDN_UPLOAD_COMMAND;\nif (!baseUrl) {{\n  throw new Error('ZR_CDN_BASE_URL is required before publishing the {} export');\n}}\nif (!uploadCommand) {{\n  throw new Error('ZR_CDN_UPLOAD_COMMAND is required before publishing the {} export');\n}}\nconst output = join('dist', '{}');\nawait access(output);\nconst entries = [];\nasync function collectFiles(root) {{\n  for (const entry of await readdir(root, {{ withFileTypes: true }})) {{\n    const path = join(root, entry.name);\n    if (entry.isDirectory()) {{\n      await collectFiles(path);\n    }} else if (!path.endsWith('.br') && !path.endsWith('.gz')) {{\n      entries.push(path);\n    }}\n  }}\n}}\nawait collectFiles(output);\nconst manifest = [];\nfor (const path of entries) {{\n  const bytes = await readFile(path);\n  const integrity = 'sha256-' + createHash('sha256').update(bytes).digest('base64');\n  await writeFile(path + '.br', await brotli(bytes));\n  await writeFile(path + '.gz', await gzipAsync(bytes));\n  manifest.push({{ path: relative(output, path).replaceAll('\\\\', '/'), bytes: bytes.length, integrity }});\n}}\nawait writeFile(join(output, 'zircon-export.integrity.json'), JSON.stringify({{ baseUrl, manifest }}, null, 2));\nawait execFileAsync(uploadCommand, [output, baseUrl], {{ shell: true }});\nconsole.log(`CDN publish completed for ${{output}} -> ${{baseUrl}}`);\n",
        javascript_string_escape(host_name),
        javascript_string_escape(host_name),
        javascript_string_escape(host_name)
    )
}

fn browser_readme_template(profile: &ExportProfile, title: &str) -> String {
    format!(
        "# {title}\n\nProfile `{}` targets browser resources through fetch and static or VM plugin packaging. Run `npm install`, `npm run build`, and `npm run package:export` from this folder after compiling the Rust `cdylib` to `zircon_export_{}.wasm`. The generated `public/zircon-export.manifest.json` records the fetch contract, and the Vite config keeps COOP/COEP headers enabled for WebGPU and threaded WASM hosts.\n",
        profile.name,
        native_library_stem(&profile.output_name)
    )
}

fn browser_index_template(profile: &ExportProfile, script_name: &str) -> String {
    format!(
        "<!doctype html>\n<html lang=\"en\">\n<head>\n    <meta charset=\"utf-8\" />\n    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\" />\n    <title>{}</title>\n</head>\n<body>\n    <canvas id=\"zircon-canvas\" style=\"touch-action: none\"></canvas>\n    <script type=\"module\" src=\"./{}\"></script>\n</body>\n</html>\n",
        html_escape(&profile.output_name),
        script_name
    )
}

fn webgpu_host_script_template(profile: &ExportProfile) -> String {
    browser_runtime_host_script_template("webgpu", Some(&profile.name))
}

fn wasm_host_script_template(_profile: &ExportProfile) -> String {
    browser_runtime_host_script_template("wasm", None)
}

fn browser_runtime_host_script_template(host_name: &str, profile_name: Option<&str>) -> String {
    let gpu_checks = profile_name
        .map(|profile_name| {
            format!(
                "if (!navigator.gpu) {{\n    throw new Error('WebGPU is unavailable for Zircon export profile {}');\n}}\nconst adapter = await navigator.gpu.requestAdapter();\nif (!adapter) {{\n    throw new Error('WebGPU adapter is unavailable for Zircon export profile {}');\n}}\n",
                javascript_string_escape(profile_name),
                javascript_string_escape(profile_name)
            )
        })
        .unwrap_or_else(|| "const adapter = null;\n".to_string());
    let mut script = String::with_capacity(10_240);
    script.push_str(
        r#"const canvas = document.querySelector('#zircon-canvas');
const manifest = await fetch('./zircon-export.manifest.json').then((response) => response.json());
const zirconExportImports = {
    env: {
        zircon_host_fetch_resource: (uriPtr, uriLen, flags) => {
            console.warn('Zircon host fetch ABI callback requires generated memory adapter', uriPtr, uriLen, flags);
            return 0;
        }
    }
};
const { instance: wasmInstance } = await WebAssembly.instantiateStreaming(fetch(manifest.wasmModule), zirconExportImports);
const zirconRuntimeExports = wasmInstance.exports;
"#,
    );
    script.push_str(&gpu_checks);
    script.push_str(
        r#"const zirconExportPendingPointerMoves = new Map();
const zirconExportActivePointers = new Map();
const zirconExportInputTelemetry = {
    inputEventsReceived: 0,
    inputEventsCoalesced: 0,
    inputEventsDispatched: 0,
    inputRawDeltaX: 0,
    inputRawDeltaY: 0,
    maxInputQueueAge: 0,
    inputAbiWallTime: 0,
    inputMainThreadWallTime: 0,
};
let zirconExportPendingViewportMetrics = null;
let zirconExportFrameRequest = null;

function zirconExportLifecycleCode(state) {
    return state === 'resumed' ? 4 : state === 'suspended' ? 8 : 0;
}
function zirconExportPointerPhaseCode(phase) {
    return phase === 'started' ? 1 : phase === 'moved' ? 2 : phase === 'ended' ? 3 : phase === 'cancelled' ? 4 : 0;
}
function zirconExportKeyActionCode(action) {
    return action === 'pressed' ? 1 : action === 'released' ? 2 : 0;
}
function zirconExportMeasureAbi(operation) {
    const startedAt = performance.now();
    operation();
    zirconExportInputTelemetry.inputAbiWallTime += performance.now() - startedAt;
}
function zirconExportMeasureMainThread(operation) {
    const startedAt = performance.now();
    try {
        return operation();
    } finally {
        zirconExportInputTelemetry.inputMainThreadWallTime += performance.now() - startedAt;
    }
}
function zirconExportDispatchLifecycle(state) {
    window.zirconRuntime?.handleLifecycle?.(state);
    zirconRuntimeExports.zircon_export_handle_lifecycle?.(zirconExportLifecycleCode(state));
}
function zirconExportDispatchPointer(pointerId, phase, x, y) {
    window.zirconRuntime?.handleTouch?.({ pointerId, phase, x, y });
    phase = zirconExportPointerPhaseCode(phase);
    zirconRuntimeExports.zircon_export_handle_touch?.(BigInt(pointerId), phase, x, y);
}
function zirconExportDispatchKeyboard(action, code, text) {
    window.zirconRuntime?.handleKeyboard?.({ action, code, text });
    zirconRuntimeExports.zircon_export_handle_keyboard?.(zirconExportKeyActionCode(action), 0, 0, 0, 0);
}
async function zirconExportFetchResource(uri, { streaming = false } = {}) {
    const url = new URL(uri, location.href);
    if (!url.pathname.startsWith(new URL(manifest.allowedAssetRoot, location.href).pathname)) {
        throw new Error(`Blocked Zircon resource fetch outside ${manifest.allowedAssetRoot}: ${uri}`);
    }
    const response = await fetch(url);
    return streaming ? response.body : new Uint8Array(await response.arrayBuffer());
}
function zirconExportDispatchViewportMetrics() {
    const rect = canvas.getBoundingClientRect();
    window.zirconRuntime?.handleViewportMetrics?.({ width: rect.width, height: rect.height, scale: window.devicePixelRatio || 1 });
    zirconRuntimeExports.zircon_export_handle_viewport_metrics?.(Math.trunc(rect.width), Math.trunc(rect.height), window.devicePixelRatio || 1);
}
function zirconExportScheduleFrameInput() {
    if (zirconExportFrameRequest === null) {
        zirconExportFrameRequest = requestAnimationFrame((now) =>
            zirconExportMeasureMainThread(() => zirconExportFlushFrameInput(now)),
        );
    }
}
function zirconExportQueuePointerMove(event) {
    const coalescedEvents = event.getCoalescedEvents?.();
    const samples = coalescedEvents?.length ? coalescedEvents : [event];
    let pending = zirconExportPendingPointerMoves.get(event.pointerId);
    for (const sample of samples) {
        zirconExportInputTelemetry.inputEventsReceived += 1;
        if (pending) {
            zirconExportInputTelemetry.inputEventsCoalesced += 1;
        }
        pending = {
            latestEvent: sample,
            deltaX: (pending?.deltaX ?? 0) + (sample.movementX ?? 0),
            deltaY: (pending?.deltaY ?? 0) + (sample.movementY ?? 0),
            queuedAt: pending?.queuedAt ?? performance.now(),
        };
    }
    zirconExportPendingPointerMoves.set(event.pointerId, pending);
    if (zirconExportActivePointers.has(event.pointerId)) {
        zirconExportActivePointers.set(event.pointerId, {
            pointerId: event.pointerId,
            clientX: pending.latestEvent.clientX,
            clientY: pending.latestEvent.clientY,
        });
    }
    zirconExportScheduleFrameInput();
}
function zirconExportFlushPointerMove(pointerId, now = performance.now()) {
    const pending = zirconExportPendingPointerMoves.get(pointerId);
    if (!pending) {
        return;
    }
    zirconExportPendingPointerMoves.delete(pointerId);
    const event = pending.latestEvent;
    zirconExportMeasureAbi(() => {
        zirconExportDispatchPointer(event.pointerId, 'moved', event.clientX, event.clientY);
    });
    window.zirconRuntime?.handlePointerMotion?.({
        pointerId: event.pointerId,
        deltaX: pending.deltaX,
        deltaY: pending.deltaY,
    });
    zirconExportInputTelemetry.inputRawDeltaX += pending.deltaX;
    zirconExportInputTelemetry.inputRawDeltaY += pending.deltaY;
    zirconExportInputTelemetry.inputEventsDispatched += 1;
    zirconExportInputTelemetry.maxInputQueueAge = Math.max(
        zirconExportInputTelemetry.maxInputQueueAge,
        now - pending.queuedAt,
    );
}
function zirconExportDrainFrameInput() {
    if (zirconExportFrameRequest !== null) {
        cancelAnimationFrame(zirconExportFrameRequest);
    }
    zirconExportFlushFrameInput();
}
function zirconExportDispatchPointerStart(event) {
    zirconExportDrainFrameInput();
    zirconExportActivePointers.set(event.pointerId, {
        pointerId: event.pointerId,
        clientX: event.clientX,
        clientY: event.clientY,
    });
    canvas.setPointerCapture?.(event.pointerId);
    zirconExportInputTelemetry.inputEventsReceived += 1;
    zirconExportMeasureAbi(() => {
        zirconExportDispatchPointer(event.pointerId, 'started', event.clientX, event.clientY);
    });
    zirconExportInputTelemetry.inputEventsDispatched += 1;
}
function zirconExportDispatchPointerEnd(event, phase) {
    if (!zirconExportActivePointers.has(event.pointerId)) {
        return;
    }
    zirconExportDrainFrameInput();
    zirconExportActivePointers.delete(event.pointerId);
    zirconExportInputTelemetry.inputEventsReceived += 1;
    zirconExportMeasureAbi(() => {
        zirconExportDispatchPointer(event.pointerId, phase, event.clientX, event.clientY);
    });
    zirconExportInputTelemetry.inputEventsDispatched += 1;
    if (canvas.hasPointerCapture?.(event.pointerId)) {
        canvas.releasePointerCapture(event.pointerId);
    }
}
function zirconExportCancelActivePointers() {
    while (zirconExportActivePointers.size > 0) {
        const event = zirconExportActivePointers.values().next().value;
        zirconExportDispatchPointerEnd(event, 'cancelled');
    }
}
function zirconExportQueueViewportMetrics() {
    zirconExportInputTelemetry.inputEventsReceived += 1;
    if (zirconExportPendingViewportMetrics) {
        zirconExportInputTelemetry.inputEventsCoalesced += 1;
    }
    zirconExportPendingViewportMetrics = {
        queuedAt: zirconExportPendingViewportMetrics?.queuedAt ?? performance.now(),
    };
    zirconExportScheduleFrameInput();
}
function zirconExportFlushViewportMetrics(now = performance.now()) {
    const pending = zirconExportPendingViewportMetrics;
    if (!pending) {
        return;
    }
    zirconExportPendingViewportMetrics = null;
    zirconExportMeasureAbi(zirconExportDispatchViewportMetrics);
    zirconExportInputTelemetry.inputEventsDispatched += 1;
    zirconExportInputTelemetry.maxInputQueueAge = Math.max(
        zirconExportInputTelemetry.maxInputQueueAge,
        now - pending.queuedAt,
    );
}
function zirconExportFlushFrameInput(now = performance.now()) {
    zirconExportFrameRequest = null;
    for (const pointerId of zirconExportPendingPointerMoves.keys()) {
        zirconExportFlushPointerMove(pointerId, now);
    }
    zirconExportFlushViewportMetrics(now);
}
canvas.addEventListener('pointermove', (event) => {
    zirconExportMeasureMainThread(() => zirconExportQueuePointerMove(event));
});
canvas.addEventListener('pointerdown', (event) => {
    zirconExportMeasureMainThread(() => zirconExportDispatchPointerStart(event));
});
canvas.addEventListener('lostpointercapture', (event) => {
    zirconExportMeasureMainThread(() => zirconExportDispatchPointerEnd(event, 'cancelled'));
});
window.addEventListener('pointerup', (event) => {
    zirconExportMeasureMainThread(() => zirconExportDispatchPointerEnd(event, 'ended'));
});
window.addEventListener('pointercancel', (event) => {
    zirconExportMeasureMainThread(() => zirconExportDispatchPointerEnd(event, 'cancelled'));
});
window.addEventListener('keydown', (event) => {
    zirconExportMeasureMainThread(() => {
        zirconExportDrainFrameInput();
        zirconExportInputTelemetry.inputEventsReceived += 1;
        zirconExportMeasureAbi(() => zirconExportDispatchKeyboard('pressed', event.code, event.key));
        zirconExportInputTelemetry.inputEventsDispatched += 1;
    });
});
window.addEventListener('keyup', (event) => {
    zirconExportMeasureMainThread(() => {
        zirconExportDrainFrameInput();
        zirconExportInputTelemetry.inputEventsReceived += 1;
        zirconExportMeasureAbi(() => zirconExportDispatchKeyboard('released', event.code, event.key));
        zirconExportInputTelemetry.inputEventsDispatched += 1;
    });
});
window.addEventListener('resize', () => {
    zirconExportMeasureMainThread(zirconExportQueueViewportMetrics);
});
window.addEventListener('pageshow', () => {
    zirconExportMeasureMainThread(() => zirconExportDispatchLifecycle('resumed'));
});
window.addEventListener('pagehide', () => {
    zirconExportMeasureMainThread(() => {
        zirconExportDrainFrameInput();
        zirconExportCancelActivePointers();
        zirconExportDispatchLifecycle('suspended');
    });
});
zirconRuntimeExports.zircon_export_start?.();
zirconExportDispatchLifecycle('resumed');
zirconExportDispatchViewportMetrics();
window.zirconExportHost = {
    target: '"#,
    );
    script.push_str(&javascript_string_escape(host_name));
    script.push_str(
        r#"',
    canvas,
    adapter,
    manifest,
    wasmInstance,
    runtimeExports: zirconRuntimeExports,
    inputTelemetry: zirconExportInputTelemetry,
    flushInput: () => zirconExportMeasureMainThread(zirconExportFlushFrameInput),
    resourceManifest: './assets/zircon-project.toml',
    fetchResource: zirconExportFetchResource,
};
window.addEventListener('pagehide', (event) => {
    if (!event.persisted) {
        zirconRuntimeExports.zircon_export_shutdown?.();
    }
});
"#,
    );
    script
}
