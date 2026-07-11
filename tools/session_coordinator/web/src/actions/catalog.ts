export const riskLabel = { green: "只读", yellow: "受控变更", red: "高风险" } as const;
export const roleRank = { observer: 0, operator: 1, committer: 2, maintainer: 3 } as const;

export function canUseAction(
  role: keyof typeof roleRank,
  requiredRole: keyof typeof roleRank,
  enabled: boolean,
): boolean {
  return enabled && roleRank[role] >= roleRank[requiredRole];
}
