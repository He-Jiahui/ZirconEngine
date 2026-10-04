// 消费后端提供的本地化数量模板，模板约定一个数量占位；显示归一化不改变原始统计或选择复数规则。
export function formatCountText(template: string, count: number): string {
  const normalizedCount = Number.isFinite(count) ? Math.max(0, Math.trunc(count)) : 0;
  return template.replace("{count}", `${normalizedCount}`);
}
