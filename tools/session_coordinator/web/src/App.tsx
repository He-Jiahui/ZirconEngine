import { Alert, AppBar, Box, CircularProgress, CssBaseline, Drawer, IconButton, List, ListItemButton, ListItemText, Stack, Toolbar, Typography } from "@mui/material";
import { ThemeProvider } from "@mui/material/styles";
import { lazy, Suspense, useEffect, useState } from "react";
import { actionClient } from "./actions/actionClient";
import type { ControlAuthSession } from "./api/contracts";
import { StatusText } from "./components/StatusText";
import { routeForPath, routes, type RouteKey } from "./navigation";
import { ControlStoreProvider, useControlStore } from "./state/controlStore";
import { controlTheme, controlTokens } from "./theme";
import "./styles.css";

const drawerWidth = 222;
const OverviewPage = lazy(() => import("./pages/OverviewPage").then((module) => ({ default: module.OverviewPage })));
const WorkflowsPage = lazy(() => import("./pages/WorkflowsPage").then((module) => ({ default: module.WorkflowsPage })));
const SessionsPage = lazy(() => import("./pages/SessionsPage").then((module) => ({ default: module.SessionsPage })));
const ActionsPage = lazy(() => import("./pages/ActionsPage").then((module) => ({ default: module.ActionsPage })));
const FailuresPage = lazy(() => import("./pages/FailuresPage").then((module) => ({ default: module.FailuresPage })));
const CollaborationPage = lazy(() => import("./pages/CollaborationPage").then((module) => ({ default: module.CollaborationPage })));
const ValidationPage = lazy(() => import("./pages/ValidationPage").then((module) => ({ default: module.ValidationPage })));
const GitPage = lazy(() => import("./pages/GitPage").then((module) => ({ default: module.GitPage })));
const AuditPage = lazy(() => import("./pages/AuditPage").then((module) => ({ default: module.AuditPage })));
const LogsPage = lazy(() => import("./pages/LogsPage").then((module) => ({ default: module.LogsPage })));
const AboutPage = lazy(() => import("./pages/AboutPage").then((module) => ({ default: module.AboutPage })));

export default function App() {
  return <ThemeProvider theme={controlTheme}><CssBaseline /><ControlStoreProvider><ControlShell /></ControlStoreProvider></ThemeProvider>;
}

function ControlShell() {
  const state = useControlStore();
  const [route, setRoute] = useState<RouteKey>(() => routeForPath(window.location.pathname));
  const [mobileOpen, setMobileOpen] = useState(false);
  const [auth, setAuth] = useState<ControlAuthSession | null>(null);
  useEffect(() => { const update = () => setRoute(routeForPath(window.location.pathname)); window.addEventListener("popstate", update); return () => window.removeEventListener("popstate", update); }, []);
  useEffect(() => { actionClient.authSession().then(setAuth).catch(() => setAuth(null)); }, []);
  const navigate = (path: string) => { window.history.pushState({}, "", path); setRoute(routeForPath(path)); setMobileOpen(false); };
  const nav = <List component="nav" aria-label="控制中心页面">{routes.map((item) => <ListItemButton key={item.key} selected={route === item.key} onClick={() => navigate(item.path)}><ListItemText primary={item.label} /></ListItemButton>)}</List>;
  const activeSessions = state.snapshot?.sessions.filter((session) => ["active", "resolving_failure", "waiting_validation", "finalizing"].includes(session.status)).length ?? 0;
  const runningTasks = state.snapshot?.validation.cargoJobs.filter((job) => job.status === "running").length ?? 0;
  const openFailures = state.snapshot?.failures.nodes.filter((failure) => failure.status === "open").length ?? 0;

  return <Box sx={{ minHeight: "100vh" }}>
    <AppBar position="fixed" color="inherit" elevation={0} sx={{ borderBottom: `1px solid ${controlTokens.colors.lineStrong}`, zIndex: (theme) => theme.zIndex.drawer + 1 }}>
      <Toolbar><IconButton onClick={() => setMobileOpen(true)} sx={{ display: { md: "none" } }} aria-label="打开导航"><span aria-hidden="true">☰</span></IconButton><Typography variant="h6" sx={{ flex: 1 }}>Zircon 工作流控制中心</Typography>
        <Stack direction="row" spacing={1} sx={{ alignItems: "center" }}><StatusText value={state.connected ? "实时在线" : "实时重连中"} />{state.snapshot && <><StatusText value={state.snapshot.service.status} /><StatusText value={`分支 ${state.snapshot.service.branch}`} /><StatusText value={`基线 ${state.snapshot.service.baseline}`} /><Typography variant="caption">Session {activeSessions} · 任务 {runningTasks} · 告警 {openFailures}</Typography></>}<Typography variant="caption">权限 {auth?.role ?? "observer"}</Typography><IconButton onClick={state.refresh} aria-label="重新同步"><span aria-hidden="true">↻</span></IconButton></Stack>
      </Toolbar>
    </AppBar>
    <Drawer variant="permanent" sx={{ display: { xs: "none", md: "block" }, width: drawerWidth, [`& .MuiDrawer-paper`]: { width: drawerWidth, pt: "73px", background: controlTokens.colors.chrome } }}>{nav}</Drawer>
    <Drawer open={mobileOpen} onClose={() => setMobileOpen(false)} sx={{ display: { md: "none" } }} ModalProps={{ keepMounted: true }}>{nav}</Drawer>
    <Box component="main" id="main-content" tabIndex={-1} sx={{ ml: { md: `${drawerWidth}px` }, pt: "92px", px: { xs: 2, md: 3 }, pb: 4 }}>
      <Stack spacing={2} sx={{ maxWidth: 1800, mx: "auto" }}>
        <Alert severity="info">控制台只展示协调器持久化状态。业务 Session 的中间版本由服务管理，不会形成 Git 噪声提交。</Alert>
        {(state.error || state.needsRefresh) && <Alert severity={state.error ? "warning" : "info"} role="alert">{state.error ?? "状态已变化，正在重新同步完整快照。"}</Alert>}
        {state.loading && !state.snapshot && <Stack sx={{ alignItems: "center", p: 8 }}><CircularProgress aria-label="加载控制快照" /></Stack>}
        {state.snapshot && <Suspense fallback={<CircularProgress aria-label="加载控制页面" />}><CurrentPage route={route} snapshot={state.snapshot} auth={auth} onAuthChange={setAuth} /></Suspense>}
      </Stack>
    </Box>
  </Box>;
}

function CurrentPage({ route, snapshot, auth, onAuthChange }: { route: RouteKey; snapshot: NonNullable<ReturnType<typeof useControlStore>["snapshot"]>; auth: ControlAuthSession | null; onAuthChange: (session: ControlAuthSession) => void }) {
  switch (route) {
    case "overview": return <OverviewPage snapshot={snapshot} />;
    case "workflows": return <WorkflowsPage workflows={snapshot.workflows} collaboration={snapshot.collaboration} failures={snapshot.failures} />;
    case "sessions": return <SessionsPage sessions={snapshot.sessions} codexSessions={snapshot.codexSessions} />;
    case "actions": return <ActionsPage service={snapshot.service} sessions={snapshot.sessions} workflows={snapshot.workflows} auth={auth} onAuthChange={onAuthChange} />;
    case "failures": return <FailuresPage failures={snapshot.failures} />;
    case "collaboration": return <CollaborationPage collaboration={snapshot.collaboration} />;
    case "validation": return <ValidationPage validation={snapshot.validation} />;
    case "git": return <GitPage git={snapshot.git} />;
    case "audit": return <AuditPage events={snapshot.audit} />;
    case "logs": return <LogsPage events={snapshot.audit} />;
    case "about": return <AboutPage service={snapshot.service} />;
  }
}
