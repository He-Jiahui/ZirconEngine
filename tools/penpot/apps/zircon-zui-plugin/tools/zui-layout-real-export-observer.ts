import { writeFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import type { CDPSession, Locator, Page } from 'playwright';
import { requireRealSessionDirectory } from './zui-layout-real-session-contract.mjs';

type Observation = Record<string, unknown>;

async function boundedObserver<T>(label: string, work: Promise<T>): Promise<T> {
  let timer: ReturnType<typeof setTimeout> | undefined;
  try {
    return await Promise.race([work, new Promise<never>((_, reject) => {
      timer = setTimeout(() => reject(new Error(`${label} timed out after 5 seconds`)), 5_000);
    })]);
  } finally { clearTimeout(timer); }
}

/** Observe the existing trusted UI click; preserve its normal actionability and timeout. */
export async function observeActualExportClick(page: Page, button: Locator, output: string,
  receipt: Observation): Promise<void> {
  requireRealSessionDirectory(output);
  const events: Observation[] = [];
  receipt['exportBrowserEvents'] = events;
  await page.exposeBinding('zirconExportObservation', (_source, payload: unknown) => {
    if (payload && typeof payload === 'object' && !Array.isArray(payload)) {
      events.push({ ...payload as Observation, receivedAtUtc: new Date().toISOString() });
    }
  });
  await button.evaluate((element) => {
    const exposed = window as unknown as {
      zirconExportObservation: (payload: Record<string, unknown>) => Promise<void>;
    };
    const notify = (payload: Record<string, unknown>): void => {
      void exposed.zirconExportObservation(payload).catch(() => undefined);
    };
    element.addEventListener('click', (event) => notify({ kind: 'actual-button-click',
      trusted: event.isTrusted, browserTimestamp: event.timeStamp, browserAtUtc: new Date().toISOString(),
      disabled: (element as HTMLButtonElement).disabled }), { capture: true, once: true, passive: true });
    window.addEventListener('message', (event) => {
      const data: unknown = event.data;
      if (event.source !== window.parent || !data || typeof data !== 'object' || Array.isArray(data)) return;
      const message = data as Record<string, unknown>;
      if (!['selection', 'theme', 'status', 'export-ready'].includes(String(message['type']))) return;
      notify({ kind: 'actual-host-message', type: message['type'], browserAtUtc: new Date().toISOString(),
        level: message['level'], message: typeof message['message'] === 'string' ? message['message'].slice(0, 1024) : undefined,
        label: message['label'], canExport: message['canExport'],
        exportSourceUtf16Length: typeof message['source'] === 'string' ? message['source'].length : undefined });
    }, { passive: true });
  });
  let profiler: CDPSession | undefined;
  let started = false;
  receipt['exportCpuProfilerStarted'] = false;
  try {
    profiler = await boundedObserver('Profiler session setup', page.context().newCDPSession(page));
    await boundedObserver('Profiler.enable', profiler.send('Profiler.enable'));
    await boundedObserver('Profiler.start', profiler.send('Profiler.start'));
    started = true;
    receipt['exportCpuProfilerStarted'] = true;
    receipt['exportCpuProfilerStartedAtUtc'] = new Date().toISOString();
  } catch (error) { receipt['exportCpuProfilerSetupError'] = String(error); }
  receipt['exportClickRequestedAtUtc'] = new Date().toISOString();
  try {
    await button.click({ timeout: 30_000 });
    receipt['exportClickCompletedAtUtc'] = new Date().toISOString();
  } catch (error) {
    // This error is exclusively the UI Locator.click call log; it has no API request headers.
    receipt['exportClickActionabilityFailure'] = String(error);
    throw error;
  } finally {
    try {
      if (started && profiler) {
        const result = await boundedObserver('Profiler.stop', profiler.send('Profiler.stop'));
        const profilePath = resolve(output, 'export-click-cpu-profile.json');
        await writeFile(profilePath, `${JSON.stringify(result.profile)}\n`);
        receipt['exportCpuProfilePath'] = profilePath;
      }
    } catch (error) { receipt['exportCpuProfilerStopError'] = String(error); }
    finally {
      if (profiler) {
        try { await boundedObserver('Profiler.detach', profiler.detach()); }
        catch (error) { receipt['exportCpuProfilerDetachError'] = String(error); }
      }
    }
  }
}
