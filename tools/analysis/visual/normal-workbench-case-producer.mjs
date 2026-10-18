import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { lstat, readFile, readdir, realpath, writeFile } from 'node:fs/promises';
import { dirname, join, relative, resolve, sep } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const sourcePath = 'zircon_editor/assets/ui/editor/unreal/level_editor_window.zui';
const canonicalRepo = 'E:/Git/ZirconEngine';
const cursorFields = ['structure', 'geometry', 'interaction', 'viewport', 'hitTest', 'theme', 'diagnostics'];
const sha = (bytes) => createHash('sha256').update(bytes).digest('hex');
const record = (value) => value !== null && typeof value === 'object' && !Array.isArray(value);
const uint = (value) => Number.isSafeInteger(value) && value >= 0;
const jsonBytes = (value) => Buffer.from(JSON.stringify(value, null, 2) + '\n');
const physicalKey = (path) => path.replaceAll('/', '\\').replace(/^\\\\\?\\/, '').replace(/[\\]+$/, '').toLowerCase();
const requireValue = (condition, reason) => { if (!condition) throw new Error(reason); };

async function unaliased(path) {
  const absolute = resolve(path);
  let ancestor = absolute;
  for (;;) {
    requireValue(!(await lstat(ancestor)).isSymbolicLink(), `Path contains a reparse alias: ${path}`);
    const parent = dirname(ancestor);
    if (parent === ancestor) break;
    ancestor = parent;
  }
  const physical = await realpath(absolute);
  requireValue(physicalKey(physical) === physicalKey(absolute), `Path is not its physical location: ${path}`);
  return absolute;
}

function actualCursor(value, label) {
  requireValue(record(value) && Object.keys(value).length === cursorFields.length &&
    cursorFields.every((field) => Object.hasOwn(value, field) && uint(value[field])),
  `${label} requires the exact seven actual safe counters`);
  return value;
}

function actualNormalSeed(profile) {
  const evidence = profile.normal_window_evidence;
  const publication = evidence?.workbenchStatePublication;
  requireValue(profile.presenter_backend === 'gpu' && uint(profile.capture_sequence) && profile.capture_sequence > 0 &&
    record(evidence) && evidence.captureSequence === profile.capture_sequence && record(publication) &&
    publication.authority === 'normal-retained-workbench-projection' && publication.observed === true,
  'Pending: no observed actual successful normal GPU business publication; no case was synthesized');
  const cursor = actualCursor(publication.presentationCursor, 'Business publication');
  actualCursor(evidence.presentationCursor, 'Normal packet');
  assert.deepEqual(evidence.presentationCursor, cursor, 'Actual normal/business packet cursors differ');
  requireValue(uint(publication.presentedFrameCount) && publication.presentedFrameCount > 0 &&
    publication.presentedFrameCount === evidence.presentedFrameCount &&
    publication.presentedFrameCount === profile.submitted_gpu_text?.presentedFrameCount &&
    evidence.commandStreamFullRebuild === true && profile.submitted_gpu_text?.commandStreamFullRebuild === true &&
    profile.submitted_gpu_text?.coverage === 'submitted-command-stream',
  'Pending: actual business DTO does not belong to a complete successfully submitted normal GPU frame');
  return { evidence, publication, cursor };
}

async function currentContractInputs(modulePath, repo) {
  const auditPath = resolve(dirname(modulePath), 'normal-workbench-contract-inputs.json');
  await unaliased(modulePath);
  await unaliased(auditPath);
  const bytes = await readFile(auditPath);
  const audit = JSON.parse(bytes);
  requireValue(audit.schema === 'dev.zircon.editor.normal-workbench-contract-inputs' && audit.version === 1 &&
    physicalKey(audit.repoRoot) === physicalKey(repo) && Array.isArray(audit.inputs) && audit.inputs.length > 0 &&
    /^[0-9a-f]{64}$/.test(audit.outputSha256), 'Canonical contract input audit is missing or unsupported');
  requireValue(sha(await readFile(modulePath)) === audit.outputSha256, 'Compiled canonical contract changed');
  const paths = new Set();
  const canonicalPrefix = resolve(repo, 'dev/penpot/plugins/apps/zircon-zui-plugin') + sep;
  for (const input of audit.inputs) {
    requireValue(record(input) && typeof input.path === 'string' && /^[0-9a-f]{64}$/.test(input.sha256),
      'Canonical contract input identity is malformed');
    const path = await unaliased(input.path);
    requireValue(!paths.has(physicalKey(path)), 'Canonical contract input audit contains duplicates');
    paths.add(physicalKey(path));
    requireValue(sha(await readFile(path)) === input.sha256, `Canonical contract input changed: ${path}`);
  }
  for (const file of ['tools/zui-layout-review-contract.ts', 'tools/zui-layout-workbench-presentation.ts',
    'tools/zui-layout-workbench-managed-inputs.ts', 'tools/zui-layout-component-hosts.ts']) {
    requireValue(paths.has(physicalKey(canonicalPrefix + file)), `Canonical contract owner missing: ${file}`);
  }
  return { audit, auditPath, auditSha256: sha(bytes), moduleSha256: audit.outputSha256 };
}

async function sourceInputs(repo) {
  const paths = [];
  async function scan(directory) {
    for (const entry of await readdir(directory, { withFileTypes: true })) {
      requireValue(!entry.isSymbolicLink(), 'Source closure contains a reparse alias');
      const path = join(directory, entry.name);
      if (entry.isDirectory()) await scan(path);
      else if (entry.isFile() && entry.name.endsWith('.zui')) paths.push(relative(repo, path).replaceAll('\\', '/'));
    }
  }
  await unaliased(resolve(repo, 'zircon_editor/assets/ui'));
  await scan(resolve(repo, 'zircon_editor/assets/ui'));
  for (const dependency of ['zircon_editor/assets/i18n/en.toml',
    'zircon_editor/assets/i18n/zh-CN.toml', 'zircon_runtime/assets/fonts/editor-ui.ttc']) {
    await unaliased(resolve(repo, dependency));
    paths.push(dependency);
  }
  paths.sort();
  const inputs = [];
  for (const path of paths) inputs.push({ path, sha256: sha(await readFile(resolve(repo, path))) });
  return inputs;
}

async function main() {
  const usage = 'normal-workbench-case-producer.mjs --repo-root <actual repo> --profile <immutable actual normal GPU profile> --case-id <capture CaseId> --output-dir <existing approved artifact directory> --contract-module <current producer-sibling contracts.mjs>';
  const keys = ['--repo-root', '--profile', '--case-id', '--output-dir', '--contract-module'];
  const args = process.argv.slice(2);
  if (args.length === 1 && args[0] === '--help') { console.log(usage); return; }
  const options = new Map();
  requireValue(args.length === keys.length * 2, usage);
  for (let index = 0; index < args.length; index += 2) {
    requireValue(keys.includes(args[index]) && args[index + 1] && !options.has(args[index]), usage);
    options.set(args[index], args[index + 1]);
  }
  const repo = await unaliased(options.get('--repo-root'));
  requireValue(physicalKey(repo) === physicalKey(resolve(canonicalRepo)), 'Use the actual reviewed repository root');
  const modulePath = resolve(options.get('--contract-module'));
  requireValue(modulePath === resolve(dirname(fileURLToPath(import.meta.url)), 'normal-workbench-contracts.mjs'),
    'Contract module must be the reviewed producer sibling');
  const contractPin = await currentContractInputs(modulePath, repo);
  const contracts = await import(pathToFileURL(modulePath));
  const output = await unaliased(contracts.requireApprovedPenpotArtifactDirectory(options.get('--output-dir'), 'normal case output'));
  const profilePath = await unaliased(contracts.requireApprovedPenpotArtifactFile(options.get('--profile'), 'actual normal profile'));
  const profileBytes = await readFile(profilePath);
  requireValue(profileBytes.length <= 8 * 1024 * 1024, 'Actual normal profile exceeds bounded metadata size');
  const profile = JSON.parse(profileBytes);
  const { evidence, publication, cursor } = actualNormalSeed(profile);
  requireValue(contracts.readViewportToolbarPublication(evidence) !== undefined,
    'Pending: actual same-frame viewport toolbar publication is absent; no state/overflow was synthesized');
  const presentation = publication.workbenchPresentation;
  // The canonical validator returns void. Preserve the original observed DTO after validation.
  contracts.validateWorkbenchPresentation(presentation);
  const managed = await contracts.verifyWorkbenchManagedSceneFingerprint(publication.managedSceneFingerprint);
  contracts.assertWorkbenchManagedProjectPath(managed, presentation.status.projectPath);
  const sourceBytes = await readFile(resolve(repo, sourcePath));
  requireValue(presentation.sourceFingerprint.sourcePath === sourcePath &&
    presentation.sourceFingerprint.sha256 === sha(sourceBytes),
  'Current Unreal source owner differs from the actual normal business publication');
  const logical = profile.window_client_logical_size;
  const physical = profile.window_client_size;
  const dpi = profile.winit_scale_factor;
  requireValue(record(logical) && record(physical) && Number.isFinite(dpi) && dpi > 0 &&
    [logical.width, logical.height].every((value) => Number.isInteger(value) && value >= 1 && value <= 8192) &&
    [physical.width, physical.height].every((value) => uint(value) && value > 0) &&
    Math.abs(logical.width * dpi - physical.width) <= 0.51 &&
    Math.abs(logical.height * dpi - physical.height) <= 0.51,
  'Pending: actual viewport/DPI is not faithfully representable by LayoutReviewCase');
  const stableBusiness = { authority: publication.authority, observed: true, presentationCursor: cursor,
    presentedFrameCount: publication.presentedFrameCount,
    workbenchPresentation: presentation, managedSceneFingerprint: managed };
  const stableEvidence = { workbenchStatePublication: stableBusiness,
    presentationCursor: evidence.presentationCursor, presentedFrameCount: evidence.presentedFrameCount };
  // Forward the same successful immutable present; the reader admits actual owner focus or keeps unknown.
  if (Object.hasOwn(evidence, 'semanticNodes')) stableEvidence.semanticNodes = evidence.semanticNodes;
  // Preserve the exact observed source subtree, all7 cursor and successful frame.
  stableEvidence.viewportToolbarPublication = evidence.viewportToolbarPublication;
  // Missing focus remains unknown. Do not turn absent observations into known unfocused state.
  if (Object.hasOwn(evidence, 'focusPublication')) stableEvidence.focusPublication = evidence.focusPublication;
  // Keep the exact observations from this successful present in the hashed case.
  if (record(evidence.paintEvidence) && Object.hasOwn(evidence.paintEvidence, 'designTokenInput')) {
    stableEvidence.designTokenInput = evidence.paintEvidence.designTokenInput;
  }
  if (Object.hasOwn(evidence, 'arrangedInventoryPublication')) {
    stableEvidence.arrangedInventoryPublication = evidence.arrangedInventoryPublication;
  }
  const inputSources = await sourceInputs(repo);
  const dependencies = new contracts.LayoutDependencies();
  await dependencies.load(repo, inputSources.filter((item) => item.path.endsWith('.zui')).map((item) => item.path));
  const source = sourceBytes.toString('utf8');
  const document = contracts.parseZuiLayoutSource(source, sourcePath).document;
  const rootId = document.root?.node;
  const rootControl = document.nodes?.[rootId]?.control_id;
  requireValue(typeof rootId === 'string' && typeof rootControl === 'string' && rootControl.length > 0,
    'Actual Unreal source has no addressable root selector');
  const host = await contracts.prepareComponentReviewHost(repo, sourcePath, document, dependencies, undefined, presentation);
  requireValue(host && typeof host.source === 'string' && record(host.projection),
    'Current authored Unreal source has no supported complete host preparation');
  assert.deepEqual(host.projection.penpot_review_workbench_presentation, presentation,
    'Prepared host must carry the exact observed business DTO');
  const hostPath = resolve(output, 'normal-workbench-review-host.zui');
  const casePath = resolve(output, 'normal-workbench-case.json');
  const receiptPath = resolve(output, 'normal-workbench-case-receipt.json');
  const bindingsPath = resolve(output, 'normal-workbench-case-bindings.json');
  const reviewCase = contracts.validateLayoutReviewCase({ id: options.get('--case-id'), sourcePath, host: 'editor',
    viewport: { width: logical.width, height: logical.height }, dpi,
    locale: presentation.activeLocale === 'en' ? 'en-US' : presentation.activeLocale, state: 'default',
    data: { workbenchPresentation: presentation, workbenchState: { sourcePath, sourceNodeId: rootId, controlId: rootControl },
      managedSceneFingerprint: managed, normal_window_evidence: stableEvidence },
    reviewHost: { path: hostPath, sha256: sha(Buffer.from(host.source)) } });
  const caseHash = contracts.caseSha256(reviewCase);
  const caseBytes = jsonBytes(reviewCase);
  for (const input of inputSources) requireValue(sha(await readFile(resolve(repo, input.path))) === input.sha256,
    `Source changed during actual host preparation: ${input.path}`);
  requireValue(sha(await readFile(profilePath)) === sha(profileBytes), 'Immutable seed profile changed during preparation');
  await contracts.verifyWorkbenchManagedSceneFingerprint(managed);
  const afterPin = await currentContractInputs(modulePath, repo);
  assert.deepEqual(afterPin, contractPin, 'Contract input audit changed during preparation');
  const receipt = { schema: 'dev.zircon.editor.normal-workbench-case-preparation', version: 1, status: 'pending',
    reason: 'Actual normal business generation prepared. A fresh nonce frame must independently match the seven counters, business/source/managed/focus identity, full submitted frame and actual GPU/native pixels.',
    pixelFrameVerified: false, wholeUiAccepted: false, sourceCopiesCreated: false,
    caseJsonPath: casePath, caseJsonSha256: sha(caseBytes), caseSha256: caseHash,
    seedProfilePath: profilePath, seedProfileSha256: sha(profileBytes), seedCaptureSequence: profile.capture_sequence,
    seedPresentedFrameCount: publication.presentedFrameCount, presentationCursor: cursor,
    profilePath, profileSha256: sha(profileBytes), profileRole: 'seed-normal-observation-not-final-capture-proof',
    reviewHost: reviewCase.reviewHost, bindingsPath, sourceInputs: inputSources, consumers: host.consumers,
    contract: { path: modulePath, sha256: contractPin.moduleSha256, inputAuditPath: contractPin.auditPath,
      inputAuditSha256: contractPin.auditSha256, inputCount: contractPin.audit.inputs.length } };
  // Create once: interrupted or failed evidence remains on disk for diagnosis and is never overwritten.
  await writeFile(hostPath, host.source, { flag: 'wx', flush: true });
  await writeFile(casePath, caseBytes, { flag: 'wx', flush: true });
  await writeFile(bindingsPath, jsonBytes({ schema: 'dev.zircon.editor.normal-frame-case-bindings', version: 1,
    cases: { [reviewCase.id]: { caseJsonPath: casePath, caseJsonSha256: receipt.caseJsonSha256 } } }), { flag: 'wx', flush: true });
  await writeFile(receiptPath, jsonBytes(receipt), { flag: 'wx', flush: true });
  console.log(JSON.stringify({ status: receipt.status, receiptPath, caseJsonPath: casePath,
    caseJsonSha256: receipt.caseJsonSha256, caseSha256: caseHash, bindingsPath, profilePath,
    profileSha256: receipt.profileSha256, profileRole: receipt.profileRole, presentationCursor: cursor,
    pixelFrameVerified: false, wholeUiAccepted: false }));
}

main().catch((error) => { console.error(error.stack ?? String(error)); process.exitCode = 1; });
