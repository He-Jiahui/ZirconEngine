/**
 * A download event is observable before Chromium has finished consuming a
 * Blob URL.  Keep the URL alive for the handoff rather than revoking it in
 * the same task that activates the anchor.
 */
export const DOWNLOAD_URL_REVOCATION_DELAY_MS = 1_000;

export function deferDownloadUrlRevocation(
  url: string,
  revoke: (value: string) => void = URL.revokeObjectURL,
): void {
  setTimeout(() => revoke(url), DOWNLOAD_URL_REVOCATION_DELAY_MS);
}
