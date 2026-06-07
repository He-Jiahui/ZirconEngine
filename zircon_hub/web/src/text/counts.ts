export function formatCountText(template: string, count: number): string {
  const normalizedCount = Number.isFinite(count) ? Math.max(0, Math.trunc(count)) : 0;
  return template.replace("{count}", `${normalizedCount}`);
}
