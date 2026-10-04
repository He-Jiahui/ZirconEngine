import type { Page } from 'playwright';

export async function withPenpotStartupRetry<T>(
  open: () => Promise<T>,
  onRetry: (attempt: number, error: Error) => void,
): Promise<T> {
  for (let attempt = 1; ; attempt += 1) {
    try {
      return await open();
    } catch (error) {
      if (
        attempt >= 3 ||
        !(error instanceof Error) ||
        !error.message.includes('Timed out waiting for Penpot notifications WebSocket')
      ) throw error;
      onRetry(attempt, error);
    }
  }
}

export async function navigateWithRetry(page: Page, url: string): Promise<void> {
  let lastError: unknown;
  for (let attempt = 1; attempt <= 3; attempt += 1) {
    try {
      await page.goto(url, { waitUntil: 'commit', timeout: 120_000 });
      return;
    } catch (error) {
      lastError = error;
      if (attempt < 3) await page.waitForTimeout(1_000 * attempt);
    }
  }
  throw lastError instanceof Error
    ? lastError
    : new Error(`Unable to navigate to Penpot workspace: ${String(lastError)}`);
}
