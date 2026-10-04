import { isAbsolute, resolve } from 'node:path';
import type { BrowserRuntimeRecord } from './zui-layout-review-contract';

const BROWSER_RUNTIME_FIELDS = [
  'executablePath',
  'executableSha256',
  'product',
  'userAgent',
] as const;

export function browserRuntimeErrors(
  value: unknown,
  selectedExecutablePath: string,
): string[] {
  if (!value || typeof value !== 'object' || Array.isArray(value))
    return ['missing browser runtime receipt'];
  const record = value as Partial<BrowserRuntimeRecord> & Record<string, unknown>;
  const keys = Object.keys(record).sort();
  if (keys.join(',') !== [...BROWSER_RUNTIME_FIELDS].sort().join(','))
    return ['browser runtime receipt has an unsupported field set'];
  if (
    typeof record.product !== 'string' ||
    !/^(Chrome|Chromium|HeadlessChrome|Microsoft Edge)\/[0-9][\w. -]*$/i.test(
      record.product,
    ) ||
    typeof record.userAgent !== 'string' ||
    !/(Chrome|Chromium)\/[0-9][\w. -]*/.test(record.userAgent) ||
    typeof record.executablePath !== 'string' ||
    !isAbsolute(record.executablePath) ||
    resolve(record.executablePath) !== resolve(selectedExecutablePath) ||
    typeof record.executableSha256 !== 'string' ||
    !/^[0-9a-f]{64}$/.test(record.executableSha256)
  )
    return ['browser runtime receipt does not match the selected Playwright browser'];
  return [];
}
