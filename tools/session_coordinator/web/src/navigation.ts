export const routes = [
  { path: "/ui/", label: "总览", key: "overview" },
  { path: "/ui/workflows", label: "工作流", key: "workflows" },
  { path: "/ui/sessions", label: "会话", key: "sessions" },
  { path: "/ui/actions", label: "受控操作", key: "actions" },
  { path: "/ui/failures", label: "失败链", key: "failures" },
  { path: "/ui/collaboration", label: "协作", key: "collaboration" },
  { path: "/ui/validation", label: "验证", key: "validation" },
  { path: "/ui/git", label: "里程碑提交", key: "git" },
  { path: "/ui/audit", label: "审计", key: "audit" },
  { path: "/ui/logs", label: "日志", key: "logs" },
  { path: "/ui/about", label: "服务信息", key: "about" },
] as const;

export type RouteKey = (typeof routes)[number]["key"];

export function routeForPath(pathname: string): RouteKey {
  const exact = routes.find((route) => route.path === pathname);
  if (exact) return exact.key;
  if (pathname === "/ui") return "overview";
  const nested = routes.slice(1).find((route) => pathname.startsWith(`${route.path}/`));
  return nested?.key ?? "overview";
}
