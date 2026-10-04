import { realpathSync } from 'node:fs';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const workspace = resolve(fileURLToPath(new URL('../..', import.meta.url)));
const physical = realpathSync.native(workspace).replaceAll('\\', '/').toLowerCase();
const requested = workspace.replaceAll('\\', '/').toLowerCase();
if (
  requested !== physical ||
  !/^[def]:\/cargo-targets\/zircon-local\//.test(physical)
) {
  throw new Error('Run tools/penpot/tools/scripts/run-validation.ps1; compilation and caches require a staged workspace under D/E/F:/cargo-targets/zircon-local.');
}
