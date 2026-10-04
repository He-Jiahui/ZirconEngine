import esbuild from 'esbuild';
import { existsSync, lstatSync, mkdirSync, realpathSync, statSync } from 'fs';
import { readdir } from 'fs/promises';
import { resolve, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const rootDir = resolve(__dirname, '../..');
const appsDir = resolve(rootDir, 'apps');

const watch = process.argv.includes('--watch');
const filterPlugin = process.argv
  .find((arg) => arg.startsWith('--plugin='))
  ?.replace('--plugin=', '');

function normalizeWindowsPath(value) {
  return value
    .replaceAll('/', '\\')
    .replace(/^\\\\\?\\/, '')
    .replace(/[\\/]+$/, '')
    .toLowerCase();
}

function isWithin(path, root) {
  return path === root || path.startsWith(`${root}\\`);
}

function approvedArtifactRoot() {
  const configured = process.env.ZUI_PLUGIN_ARTIFACT_ROOT;
  if (!configured) {
    throw new Error(
      'Set ZUI_PLUGIN_ARTIFACT_ROOT to an existing directory under D:\\cargo-targets, E:\\cargo-targets, or F:\\cargo-targets before building the ZUI plugin.',
    );
  }
  const candidate = resolve(configured);
  const approvedRoots = [
    'D:\\cargo-targets',
    'E:\\cargo-targets',
    'F:\\cargo-targets',
  ];
  const lexical = normalizeWindowsPath(candidate);
  const root = approvedRoots.find((value) =>
    isWithin(lexical, normalizeWindowsPath(value)),
  );
  if (!root) {
    throw new Error(
      `ZUI_PLUGIN_ARTIFACT_ROOT must remain below an approved drive-root cargo-targets directory; received ${candidate}.`,
    );
  }
  if (!existsSync(root) || !statSync(root).isDirectory()) {
    throw new Error(`Approved artifact root does not exist: ${root}.`);
  }
  if (!existsSync(candidate) || !statSync(candidate).isDirectory()) {
    throw new Error(
      `ZUI_PLUGIN_ARTIFACT_ROOT must be an existing directory: ${candidate}.`,
    );
  }
  const physicalRoot = realpathSync.native(root);
  const physicalCandidate = realpathSync.native(candidate);
  const physicalRootPath = normalizeWindowsPath(physicalRoot);
  if (
    normalizeWindowsPath(root) !== physicalRootPath ||
    !isWithin(normalizeWindowsPath(physicalCandidate), physicalRootPath)
  ) {
    throw new Error(
      `ZUI_PLUGIN_ARTIFACT_ROOT resolves outside its approved physical root: ${candidate}.`,
    );
  }
  return candidate;
}

const artifactRoot = approvedArtifactRoot();

async function getPluginEntryPoints() {
  const entries = await readdir(appsDir, { withFileTypes: true });
  const entryPoints = [];

  for (const entry of entries) {
    if (!entry.isDirectory()) continue;

    if (filterPlugin && entry.name !== filterPlugin) continue;

    const pluginTs = resolve(appsDir, entry.name, 'src/plugin.ts');
    const tsconfigPlugin = resolve(appsDir, entry.name, 'tsconfig.plugin.json');

    if (existsSync(pluginTs) && existsSync(tsconfigPlugin)) {
      const outdir = resolve(artifactRoot, entry.name, 'bundle', 'assets');
      mkdirSync(outdir, { recursive: true });
      const physicalOutdir = normalizeWindowsPath(realpathSync.native(outdir));
      const physicalRoot = normalizeWindowsPath(
        realpathSync.native(artifactRoot),
      );
      if (!isWithin(physicalOutdir, physicalRoot)) {
        throw new Error(
          `Plugin output directory resolves outside ZUI_PLUGIN_ARTIFACT_ROOT: ${outdir}.`,
        );
      }
      const outfile = resolve(outdir, 'plugin.js');
      if (existsSync(outfile) && lstatSync(outfile).isSymbolicLink()) {
        throw new Error(
          `Refusing to write through a plugin bundle symlink: ${outfile}.`,
        );
      }
      entryPoints.push({
        name: entry.name,
        entryPoint: pluginTs,
        tsconfig: tsconfigPlugin,
        outdir,
      });
    }
  }

  return entryPoints;
}

async function buildPlugin(plugin) {
  const options = {
    entryPoints: [plugin.entryPoint],
    bundle: true,
    outfile: resolve(plugin.outdir, 'plugin.js'),
    minify: !watch,
    format: 'esm',
    tsconfig: plugin.tsconfig,
    logLevel: 'info',
  };

  if (watch) {
    const ctx = await esbuild.context(options);
    await ctx.watch();
    console.log(`[buildPlugin] Watching ${plugin.name}...`);
    return ctx;
  } else {
    await esbuild.build(options);
    console.log(`[buildPlugin] Built ${plugin.name}`);
  }
}

async function main() {
  const plugins = await getPluginEntryPoints();

  if (plugins.length === 0) {
    console.warn('[buildPlugin] No plugins found to build.');
    return;
  }

  console.log(
    `[buildPlugin] ${watch ? 'Watching' : 'Building'} ${plugins.length} plugin(s): ${plugins.map((p) => p.name).join(', ')}`,
  );

  const results = await Promise.all(plugins.map(buildPlugin));

  if (watch) {
    process.on('SIGINT', async () => {
      await Promise.all(results.map((ctx) => ctx?.dispose()));
      process.exit(0);
    });
  }
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
