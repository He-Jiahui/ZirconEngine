/**
 * Register a download listener before clicking its producer, while keeping
 * both promises observed if either side fails first.
 *
 * A sequential `waitForEvent(); await click(); await wait` flow leaks the
 * waiter when `click()` rejects.  Playwright then reports the listener's
 * timeout as an unhandled rejection after the caller has already recovered.
 */
export async function clickAndWaitForDownload<T>(
  waitForDownload: () => Promise<T>,
  click: () => Promise<unknown>,
): Promise<T> {
  const [download] = await Promise.all([waitForDownload(), click()]);
  return download;
}

/** Bound Playwright's post-event download completion wait. */
export function downloadPath(
  download: { path(): Promise<string | null> },
  timeoutMs = 30_000,
): Promise<string | null> {
  return withTimeout(
    download.path(),
    timeoutMs,
    'Timed out waiting for exported download file',
  );
}

function withTimeout<T>(
  operation: Promise<T>,
  timeoutMs: number,
  message: string,
): Promise<T> {
  return new Promise<T>((resolve, reject) => {
    const timer = setTimeout(() => reject(new Error(message)), timeoutMs);
    // Attach both handlers immediately. If the timeout wins, a late I/O
    // rejection remains consumed instead of surfacing after the batch ends.
    void operation.then(
      (value) => {
        clearTimeout(timer);
        resolve(value);
      },
      (error: unknown) => {
        clearTimeout(timer);
        reject(error);
      },
    );
  });
}
