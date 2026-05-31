import { spawn, spawnSync } from "node:child_process";
import { mkdir, writeFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { DESIGNS, HEIGHT, OUTPUT_DIR, WIDTH } from "./design-manifest.mjs";
import {
  npmConfigDesignArgs,
  parseDesignSelection,
  selectDesigns,
  shouldCapturePreviewSheet,
} from "./export-options.mjs";

const rootDir = resolve(fileURLToPath(new URL("../..", import.meta.url)));
const toolDir = resolve(rootDir, "tools/editor-workbench-preview");
const outputDir = resolve(rootDir, OUTPUT_DIR);
const port = Number.parseInt(process.env.ZIRCON_WORKBENCH_PREVIEW_PORT ?? "5187", 10);
const host = "127.0.0.1";
const baseUrl = `http://${host}:${port}`;
const designSelection = parseDesignSelection([
  ...npmConfigDesignArgs(process.env),
  ...process.argv.slice(2),
]);
const selectedDesigns = selectDesigns(DESIGNS, designSelection.ids);
const shouldCaptureSheet = shouldCapturePreviewSheet(designSelection);

await mkdir(outputDir, { recursive: true });

if (!shouldCaptureSheet && selectedDesigns.length === 0) {
  await writeStyleNote(outputDir);
} else {
  const server = spawn(process.execPath, ["server.mjs"], {
    cwd: toolDir,
    env: { ...process.env, ZIRCON_WORKBENCH_PREVIEW_PORT: String(port) },
    stdio: ["ignore", "pipe", "pipe"],
  });

  try {
    await waitForServer(baseUrl);
    if (shouldCaptureSheet) {
      const previewSheet = resolve(outputDir, "preview-sheet.png");
      runScreenshot(`${baseUrl}/design.html?design=sheet`, previewSheet);
    }

    for (const design of selectedDesigns) {
      const target = resolve(outputDir, design.output);
      runScreenshot(`${baseUrl}/design.html?design=${encodeURIComponent(design.id)}`, target);
    }

    await writeStyleNote(outputDir);
  } finally {
    await stopServer(server);
  }
}

function runScreenshot(url, target) {
  const args = [
    "playwright",
    "screenshot",
    "--channel",
    "msedge",
    "--viewport-size",
    `${WIDTH},${HEIGHT}`,
    "--wait-for-selector",
    "#design-root > *",
    "--wait-for-timeout",
    "250",
    "--timeout",
    "120000",
    url,
    target,
  ];

  const command = process.platform === "win32" ? process.execPath : "npx";
  const commandArgs =
    process.platform === "win32" ? [resolve(dirname(process.execPath), "node_modules/npm/bin/npx-cli.js"), ...args] : args;
  const result = spawnSync(command, commandArgs, {
    cwd: rootDir,
    stdio: "inherit",
  });
  if (result.status !== 0) {
    const reason = result.error ? `: ${result.error.message}` : "";
    throw new Error(`screenshot failed for ${url}${reason}`);
  }
}

async function waitForServer(url) {
  for (let i = 0; i < 80; i += 1) {
    try {
      const response = await fetch(url);
      if (response.ok) return;
    } catch {
      // Retry while the local preview server starts.
    }
    await new Promise((resolveWait) => setTimeout(resolveWait, 125));
  }
  throw new Error(`preview server did not respond at ${url}`);
}

async function stopServer(server) {
  if (server.exitCode !== null || server.signalCode !== null) {
    return;
  }

  await new Promise((resolveExit) => {
    let resolved = false;
    const finish = () => {
      if (resolved) {
        return;
      }
      resolved = true;
      clearTimeout(timeout);
      resolveExit();
    };
    const timeout = setTimeout(() => {
      if (server.exitCode === null && server.signalCode === null) {
        server.kill("SIGKILL");
      }
      finish();
    }, 1000);
    server.once("exit", () => {
      finish();
    });
    if (!server.kill()) {
      finish();
    }
  });
}

async function writeStyleNote(dir) {
  const rows = DESIGNS.map((design) => `- \`${design.output}\` - ${design.kind}`).join("\n");
  await writeFile(
    resolve(dir, "STYLE-NOTES.md"),
    `# Editor Workbench Design Notes

These PNG drafts use \`docs/ui-and-layout/workbench.png\` as the baseline density reference, then shift the editor structure toward a JetBrains-style main-tab workbench with flat modern controls.

## Output Set

${rows}
- \`preview-sheet.png\` - dense browser review sheet covering every manifest design id

## Workbench Layout Rules

- Canvas size is ${WIDTH}x${HEIGHT} for every deliverable PNG.
- UI chrome is generated from HTML/CSS so labels, controls, and panel text remain deterministic.
- Raster-style imagery is limited to viewport scenery and thumbnail-like content inside UI containers.
- Main editor pages are represented as top-level tabs, similar to Unreal editor windows inside a JetBrains-like shell: Scene Editor, Material Editor, Montage Editor, UI Asset Editor, Asset Browser, Diagnostics, and Project.
- Additional editor-page PNGs extend the same main-tab pattern to prefab, VFX, shader, terrain, audio, behavior tree, lighting bake, physics/collision, level streaming, sequencer, NavMesh/AI, render pipeline, input mapping, data table, network replication, and localization workflows.
- Production editor-page PNGs continue that pattern for visual scripting, state machines, skeletal mesh, texture inspection, material instances, prefab variants, level audit, and editor test running.
- Diagnostics/resource editor-page PNGs cover frame debugging, memory profiling, asset dependency graphs, reference finding, cook/package queues, crash session replay, log analysis, and automation reporting.
- Infrastructure/project editor-page PNGs cover layout management, theme tokens, command catalogs, module graphs, asset validation, hot reload, project history, and task boards.
- Collaboration/release editor-page PNGs cover source control, review comments, build farm, release notes, project settings, plugin development, remote devices, and session sync.
- Cinematic/animation editor-page PNGs cover cutscenes, dialogue, quests, camera rigs, control rigs, motion matching, facial animation, and blend spaces.
- World-building/environment editor-page PNGs cover foliage painting, scatter rules, volumes, weather, post process, particle libraries, collision proxies, and level variants.
- Gameplay/runtime editor-page PNGs cover abilities, effects, AI perception, spawn rules, gameplay tags, save data, world state, and telemetry dashboards.
- Platform/online editor-page PNGs cover lobbies, matchmaking, server browsing, replays, achievements, entitlements, user profiles, and online diagnostics.
- UI/UX editor-page PNGs cover HUD editing, menu flow, font atlases, icon libraries, UI data binding, accessibility audits, input prompts, and UI motion clips.
- Rendering/GPU editor-page PNGs cover shader permutations, render targets, GPU profiling, light probes, reflection captures, decals, virtual textures, and material audits.
- Audio/voice editor-page PNGs cover sound cues, audio mixing, music systems, occlusion simulation, voice banks, subtitle timing, lip sync, and audio profiling.
- Physics/simulation editor-page PNGs cover rigid bodies, constraints, destruction, cloth, vehicles, fluids, rope/cable systems, and physics profiling.
- AI/navigation editor-page PNGs cover AI directors, blackboards, EQS queries, crowd simulation, smart objects, patrol routes, cover systems, and AI profiling.
- Asset pipeline/DCC editor-page PNGs cover mesh import, LOD chains, redirect maps, texture compression queues, source asset tracing, DCC live links, metadata editing, and batch process queues.
- Engineering/production editor-page PNGs cover script editing, API browsing, plugin packaging, module settings, automation suites, build configs, cook rules, and runtime commands.
- Project governance editor-page PNGs cover asset migrations, scene diffs, prefab diffs, performance budgets, memory budgets, dependency cleanup, naming rules, and release checklists.
- Runtime QA editor-page PNGs cover gameplay debugging, replay timelines, packet inspection, latency maps, input traces, save-state diffs, repro recording, and QA triage.
- Graphics deep-dive editor-page PNGs cover render graphs, shader debugging, texture streaming, shadow maps, occlusion culling, frame comparison, material layers, and GPU memory.
- Animation production editor-page PNGs cover retargeting, IK solving, pose libraries, mocap cleanup, animation compression, root motion, event tracks, and montage debugging.
- UI diagnostics editor-page PNGs cover widget tree debugging, layout constraint solving, theme variants, localization preview, focus navigation, input glyph mapping, UI snapshot diffs, and widget performance.
- World streaming editor-page PNGs cover world partition, HLOD building, level instances, streaming profiling, scene bookmarks, spawn points, collision matrices, and environment probes.
- LiveOps editor-page PNGs cover feature flags, remote config, telemetry queries, patch planning, DLC catalogs, crash symbolication, player segments, and experiment consoles.
- Tool windows use drawer zones: Left Top for placement/prefab tools, Left Bottom for file/project trees, Right Top for hierarchy/structure, Right Bottom for properties/animation lists/details, and Bottom for output consoles, diagnostics, and timelines.
- Layout-spec PNGs are first-class references for shell structure, drawer role placement, and editor-tab-specific tool allocation.
- State-spec PNGs describe drawer collapse/expand behavior, split editors, split bottom timelines/consoles, floating tool windows, and compact workspace fallbacks.
- Content-spec PNGs describe the internal density, controls, row states, and action placement for prefab, files, hierarchy, inspector, animation list, console, timeline, and asset grid drawers.
- Overlay-spec PNGs describe common floating/editor windows: command palette, context menu, tab overflow, asset picker, import wizard, project settings, confirmation dialog, and notification center.
- Workflow-spec PNGs combine main editor tabs, drawers, bottom panels, and transient windows into practical task flows for prefab placement, asset import, shader errors, animation events, runtime debugging, build export, UI binding, and lighting bake.
- Floating-window PNGs cover heavier operational windows that sit above the workbench for preferences, keymaps, reimport conflicts, source-control submit, crash reporting, find-in-project, startup tasks, and editor updates.
- Surfaces stay near black but cleaner: \`#111416\`, \`#171a1d\`, \`#1b1f23\`, and \`#252b31\`.
- Teal \`#3cc7d6\` is reserved for active tabs, selection, focus, and key state feedback.
- Controls use rounded rectangle shapes, simple fills, 1px borders, and flat button states; avoid glossy gradients, glow, heavy shadows, and fake depth.
- Avoid nested card layouts, large hero typography, marketing composition, and AI-generated UI text.
`,
    "utf8"
  );
}
