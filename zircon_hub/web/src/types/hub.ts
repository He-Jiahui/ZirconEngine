export type StatusTone = "running" | "success" | "warning" | "error" | "neutral";

export interface HubStatusPill {
  id: string;
  label: string;
  tone: StatusTone;
}

export interface HubProjectSummary {
  id: string;
  name: string;
  path: string;
  modified: string;
  engineVersion: string;
  platform: string;
  coverId: string;
}

export interface HubRecentProject {
  id: string;
  name: string;
  engineVersion: string;
  modified: string;
  location: string;
  coverId: string;
}

export interface HubQuickAction {
  id: string;
  title: string;
  detail: string;
  icon: string;
}

export interface HubShellState {
  productName: string;
  engineVersion: string;
  activePage: string;
  taskStatus: HubStatusPill[];
  projects: HubProjectSummary[];
  recentProjects: HubRecentProject[];
  quickActions: HubQuickAction[];
}
