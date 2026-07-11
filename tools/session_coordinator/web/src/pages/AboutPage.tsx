import { List, ListItem, ListItemText } from "@mui/material";
import type { ServiceProjection } from "../api/contracts";
import { HubPanel } from "../theme";
export function AboutPage({ service }: { service: ServiceProjection }) { return <HubPanel title="本地服务信息"><List><ListItem><ListItemText primary="实例" secondary={service.instanceId} /></ListItem><ListItem><ListItemText primary="启动时间" secondary={service.startedAt} /></ListItem><ListItem><ListItemText primary="分支与模式" secondary={`${service.branch} · ${service.mode}`} /></ListItem><ListItem><ListItemText primary="API" secondary={service.controlApiVersions.join(", ")} /></ListItem></List></HubPanel>; }
