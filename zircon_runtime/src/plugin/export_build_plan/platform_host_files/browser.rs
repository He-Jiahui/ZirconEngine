use crate::plugin::{ExportProfile, ExportTargetPlatform};

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
        "<!doctype html>\n<html lang=\"en\">\n<head>\n    <meta charset=\"utf-8\" />\n    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\" />\n    <title>{}</title>\n</head>\n<body>\n    <canvas id=\"zircon-canvas\"></canvas>\n    <script type=\"module\" src=\"./{}\"></script>\n</body>\n</html>\n",
        html_escape(&profile.output_name),
        script_name
    )
}

#[allow(unreachable_code)]
fn webgpu_host_script_template(profile: &ExportProfile) -> String {
    return browser_runtime_host_script_template("webgpu", Some(&profile.name));

    format!(
        "const canvas = document.querySelector('#zircon-canvas');\nconst manifest = await fetch('./zircon-export.manifest.json').then((response) => response.json());\nif (!navigator.gpu) {{\n    throw new Error('WebGPU is unavailable for Zircon export profile {}');\n}}\nconst adapter = await navigator.gpu.requestAdapter();\nif (!adapter) {{\n    throw new Error('WebGPU adapter is unavailable for Zircon export profile {}');\n}}\nfunction zirconExportDispatchLifecycle(state) {{\n    window.zirconRuntime?.handleLifecycle?.(state);\n}}\nfunction zirconExportDispatchPointer(pointerId, phase, x, y) {{\n    window.zirconRuntime?.handleTouch?.({{ pointerId, phase, x, y }});\n}}\nfunction zirconExportDispatchKeyboard(action, code, text) {{\n    window.zirconRuntime?.handleKeyboard?.({{ action, code, text }});\n}}\nasync function zirconExportFetchResource(uri, {{ streaming = false }} = {{}}) {{\n    const response = await fetch(uri);\n    return streaming ? response.body : new Uint8Array(await response.arrayBuffer());\n}}\nfunction zirconExportDispatchViewportMetrics() {{\n    const rect = canvas.getBoundingClientRect();\n    window.zirconRuntime?.handleViewportMetrics?.({{ width: rect.width, height: rect.height, scale: window.devicePixelRatio || 1 }});\n}}\ncanvas.addEventListener('pointermove', (event) => zirconExportDispatchPointer(event.pointerId, 'moved', event.clientX, event.clientY));\nwindow.addEventListener('keydown', (event) => zirconExportDispatchKeyboard('pressed', event.code, event.key));\nwindow.addEventListener('keyup', (event) => zirconExportDispatchKeyboard('released', event.code, event.key));\nwindow.addEventListener('resize', zirconExportDispatchViewportMetrics);\nwindow.addEventListener('pageshow', () => zirconExportDispatchLifecycle('resumed'));\nwindow.addEventListener('pagehide', () => zirconExportDispatchLifecycle('suspended'));\nzirconExportDispatchLifecycle('resumed');\nzirconExportDispatchViewportMetrics();\nwindow.zirconExportHost = {{\n    target: 'web_gpu',\n    canvas,\n    adapter,\n    manifest,\n    resourceManifest: './assets/zircon-project.toml',\n    fetchResource: zirconExportFetchResource,\n}};\n",
        javascript_string_escape(&profile.name),
        javascript_string_escape(&profile.name)
    )
}

#[allow(unreachable_code)]
fn wasm_host_script_template(_profile: &ExportProfile) -> String {
    return browser_runtime_host_script_template("wasm", None);

    "const canvas = document.querySelector('#zircon-canvas');\nconst manifest = await fetch('./zircon-export.manifest.json').then((response) => response.json());\nconst wasmModule = await WebAssembly.compileStreaming(fetch(manifest.wasmModule));\nfunction zirconExportDispatchLifecycle(state) {\n    window.zirconRuntime?.handleLifecycle?.(state);\n}\nfunction zirconExportDispatchPointer(pointerId, phase, x, y) {\n    window.zirconRuntime?.handleTouch?.({ pointerId, phase, x, y });\n}\nfunction zirconExportDispatchKeyboard(action, code, text) {\n    window.zirconRuntime?.handleKeyboard?.({ action, code, text });\n}\nasync function zirconExportFetchResource(uri, { streaming = false } = {}) {\n    const response = await fetch(uri);\n    return streaming ? response.body : new Uint8Array(await response.arrayBuffer());\n}\nfunction zirconExportDispatchViewportMetrics() {\n    const rect = canvas.getBoundingClientRect();\n    window.zirconRuntime?.handleViewportMetrics?.({ width: rect.width, height: rect.height, scale: window.devicePixelRatio || 1 });\n}\ncanvas.addEventListener('pointermove', (event) => zirconExportDispatchPointer(event.pointerId, 'moved', event.clientX, event.clientY));\nwindow.addEventListener('keydown', (event) => zirconExportDispatchKeyboard('pressed', event.code, event.key));\nwindow.addEventListener('keyup', (event) => zirconExportDispatchKeyboard('released', event.code, event.key));\nwindow.addEventListener('resize', zirconExportDispatchViewportMetrics);\nwindow.addEventListener('pageshow', () => zirconExportDispatchLifecycle('resumed'));\nwindow.addEventListener('pagehide', () => zirconExportDispatchLifecycle('suspended'));\nzirconExportDispatchLifecycle('resumed');\nzirconExportDispatchViewportMetrics();\nwindow.zirconExportHost = {\n    target: 'wasm',\n    canvas,\n    manifest,\n    wasmModule,\n    resourceManifest: './assets/zircon-project.toml',\n    fetchResource: zirconExportFetchResource,\n};\n"
        .to_string()
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
    format!(
        "const canvas = document.querySelector('#zircon-canvas');\nconst manifest = await fetch('./zircon-export.manifest.json').then((response) => response.json());\nconst zirconExportImports = {{\n    env: {{\n        zircon_host_fetch_resource: (uriPtr, uriLen, flags) => {{\n            console.warn('Zircon host fetch ABI callback requires generated memory adapter', uriPtr, uriLen, flags);\n            return 0;\n        }}\n    }}\n}};\nconst {{ instance: wasmInstance }} = await WebAssembly.instantiateStreaming(fetch(manifest.wasmModule), zirconExportImports);\nconst zirconRuntimeExports = wasmInstance.exports;\n{gpu_checks}function zirconExportLifecycleCode(state) {{\n    return state === 'resumed' ? 4 : state === 'suspended' ? 8 : 0;\n}}\nfunction zirconExportPointerPhaseCode(phase) {{\n    return phase === 'started' ? 1 : phase === 'moved' ? 2 : phase === 'ended' ? 3 : phase === 'cancelled' ? 4 : 0;\n}}\nfunction zirconExportKeyActionCode(action) {{\n    return action === 'pressed' ? 1 : action === 'released' ? 2 : 0;\n}}\nfunction zirconExportDispatchLifecycle(state) {{\n    window.zirconRuntime?.handleLifecycle?.(state);\n    zirconRuntimeExports.zircon_export_handle_lifecycle?.(zirconExportLifecycleCode(state));\n}}\nfunction zirconExportDispatchPointer(pointerId, phase, x, y) {{\n    window.zirconRuntime?.handleTouch?.({{ pointerId, phase, x, y }});\n    phase = zirconExportPointerPhaseCode(phase);\n    zirconRuntimeExports.zircon_export_handle_touch?.(BigInt(pointerId), phase, x, y);\n}}\nfunction zirconExportDispatchKeyboard(action, code, text) {{\n    window.zirconRuntime?.handleKeyboard?.({{ action, code, text }});\n    zirconRuntimeExports.zircon_export_handle_keyboard?.(zirconExportKeyActionCode(action), 0, 0, 0, 0);\n}}\nasync function zirconExportFetchResource(uri, {{ streaming = false }} = {{}}) {{\n    const url = new URL(uri, location.href);\n    if (!url.pathname.startsWith(new URL(manifest.allowedAssetRoot, location.href).pathname)) {{\n        throw new Error(`Blocked Zircon resource fetch outside ${{manifest.allowedAssetRoot}}: ${{uri}}`);\n    }}\n    const response = await fetch(url);\n    return streaming ? response.body : new Uint8Array(await response.arrayBuffer());\n}}\nfunction zirconExportDispatchViewportMetrics() {{\n    const rect = canvas.getBoundingClientRect();\n    window.zirconRuntime?.handleViewportMetrics?.({{ width: rect.width, height: rect.height, scale: window.devicePixelRatio || 1 }});\n    zirconRuntimeExports.zircon_export_handle_viewport_metrics?.(Math.trunc(rect.width), Math.trunc(rect.height), window.devicePixelRatio || 1);\n}}\ncanvas.addEventListener('pointermove', (event) => zirconExportDispatchPointer(event.pointerId, 'moved', event.clientX, event.clientY));\nwindow.addEventListener('keydown', (event) => zirconExportDispatchKeyboard('pressed', event.code, event.key));\nwindow.addEventListener('keyup', (event) => zirconExportDispatchKeyboard('released', event.code, event.key));\nwindow.addEventListener('resize', zirconExportDispatchViewportMetrics);\nwindow.addEventListener('pageshow', () => zirconExportDispatchLifecycle('resumed'));\nwindow.addEventListener('pagehide', () => zirconExportDispatchLifecycle('suspended'));\nzirconRuntimeExports.zircon_export_start?.();\nzirconExportDispatchLifecycle('resumed');\nzirconExportDispatchViewportMetrics();\nwindow.zirconExportHost = {{\n    target: '{host_name}',\n    canvas,\n    adapter,\n    manifest,\n    wasmInstance,\n    runtimeExports: zirconRuntimeExports,\n    resourceManifest: './assets/zircon-project.toml',\n    fetchResource: zirconExportFetchResource,\n}};\n",
        host_name = javascript_string_escape(host_name)
    )
}
