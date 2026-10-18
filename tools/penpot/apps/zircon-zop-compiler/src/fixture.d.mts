export function fixtureBundle(screen?: Record<string, unknown>, designTokens?: Record<string, unknown>): Promise<{
  manifest: Record<string, unknown>; entries: Record<string, Uint8Array>;
}>;
