import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { compileBundle, sha256 } from './compiler.mjs';
import { generatedPath, planApply } from './apply-plan.mjs';
import { safeEntry } from './contract.mjs';

const repository = fileURLToPath(new URL('../../../../../', import.meta.url));
const landing = path.join(repository, 'zircon_editor/assets/ui/operator');

export function assertNoAlias(destination) {
  let current = path.resolve(destination);
  while (true) {
    if (fs.existsSync(current)) {
      if (fs.lstatSync(current).isSymbolicLink() || fs.realpathSync.native(current).toLowerCase() !== current.toLowerCase()) throw new Error(`path alias/reparse point rejected: ${current}`);
    }
    const parent = path.dirname(current);
    if (parent === current) break;
    current = parent;
  }
}

export function approvedArtifactRoot(destination) {
  if (process.platform !== 'win32' || !/^[DEF]:\\cargo-targets\\[^:]+$/i.test(destination)
    || destination.includes('/') || destination.split('\\').some(part => part === '.' || part === '..')) {
    throw new Error('Artifact output requires a physical drive-root D/E/F:\\cargo-targets subdirectory on Windows');
  }
  if (path.resolve(destination).toLowerCase() !== destination.toLowerCase()) throw new Error('Artifact path must be canonical');
  for (const part of destination.slice(3).split('\\')) safeEntry(part);
  assertNoAlias(destination);
  return path.resolve(destination);
}

function readTree(directory) {
  const files = {};
  if (!fs.existsSync(directory)) return files;
  assertNoAlias(directory);
  function walk(current, prefix = '') {
    for (const entry of fs.readdirSync(current, { withFileTypes: true })) {
      if (entry.isSymbolicLink()) throw new Error(`symlink input ${current}/${entry.name}`);
      const relative = prefix + entry.name, absolute = path.join(current, entry.name);
      assertNoAlias(absolute);
      if (entry.isDirectory()) walk(absolute, relative + '/');
      else if (entry.isFile()) files[relative] = fs.readFileSync(absolute);
      else throw new Error(`nonregular input ${absolute}`);
    }
  }
  walk(directory);
  return files;
}

function main() {
  const [command, ...args] = process.argv.slice(2), options = {};
  for (let i = 0; i < args.length; i += 2) {
    if (!['--bundle', '--output', '--from', '--contract', '--translations'].includes(args[i]) || !args[i + 1]) throw new Error('Usage: compile|verify --bundle <extracted-bundle> [--output <dir>]; apply --from <artifact-output>');
    if (Object.hasOwn(options, args[i])) throw new Error(`duplicate ${args[i]}`);
    options[args[i]] = args[i + 1];
  }
  if (command === 'apply') {
    if (!options['--from']) throw new Error('apply needs --from');
    const source = approvedArtifactRoot(options['--from']);
    const generated = readTree(source), existing = readTree(landing);
    const plan = planApply(existing, generated);
    // Canonical landing only, editor closed. All ownership/hash checks precede writes.
    assertNoAlias(landing);
    for (const relative of [...Object.keys(plan.writes), ...plan.deletes]) {
      if (relative !== 'zop-bundle.lock.json') generatedPath(relative);
      assertNoAlias(path.join(landing, relative));
    }
    for (const relative of plan.deletes) fs.unlinkSync(path.join(landing, relative));
    for (const [relative, bytes] of Object.entries(plan.writes)) {
      const target = path.join(landing, relative);
      fs.mkdirSync(path.dirname(target), { recursive: true });
      fs.writeFileSync(target, bytes);
    }
    console.log(`Applied ${Object.keys(plan.writes).length} files; retired ${plan.deletes.length} owned files.`);
    return;
  }
  if (!['compile', 'verify'].includes(command) || !options['--bundle']) throw new Error('compile/verify requires --bundle');
  const entries = readTree(options['--bundle']);
  const manifest = JSON.parse(new TextDecoder().decode(entries['manifest.json']));
  for (const entry of Object.keys(manifest.entries ?? {})) safeEntry(entry);
  const contract = fs.readFileSync(options['--contract'] ?? path.join(landing, 'contract/roles.contract.toml'), 'utf8');
  const translations = options['--translations'] ? JSON.parse(fs.readFileSync(options['--translations'], 'utf8'))
    : Object.fromEntries([...fs.readFileSync(path.join(repository, 'zircon_editor/assets/i18n/en.toml'), 'utf8').matchAll(/^"([^"]+)"\s*=/gm)].map(match => [match[1], true]));
  const result = compileBundle({ manifest, entries }, contract, translations);
  if (command === 'verify') {
    const existing = readTree(options['--output'] ?? landing);
    const expected = Object.keys(result.files);
    for (const relative of expected) if (!existing[relative] || sha256(existing[relative]) !== sha256(result.files[relative])) throw new Error(`generated drift: ${relative}`);
    for (const relative of Object.keys(existing)) if (/\.(zui|roles\.json|svg)$/.test(relative) && !expected.includes(relative)) throw new Error(`stale generated file: ${relative}`);
    console.log(`Verified ${expected.length} deterministic generated files.`);
    return;
  }
  const output = approvedArtifactRoot(options['--output'] ?? '');
  if (fs.existsSync(output) && fs.readdirSync(output).length) throw new Error('compile needs a new or empty artifact output directory');
  for (const [relative, bytes] of Object.entries(result.files)) {
    const target = path.join(output, relative);
    assertNoAlias(target);
    fs.mkdirSync(path.dirname(target), { recursive: true });
    fs.writeFileSync(target, bytes);
  }
  console.log(`Compiled ${Object.keys(result.files).length} files to ${output}`);
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try { main(); } catch (error) { console.error(error.message); process.exitCode = 1; }
}
