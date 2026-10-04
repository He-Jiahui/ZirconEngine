import AddOutlinedIcon from "@mui/icons-material/AddOutlined";
import CheckOutlinedIcon from "@mui/icons-material/CheckOutlined";
import CloseOutlinedIcon from "@mui/icons-material/CloseOutlined";
import GroupsOutlinedIcon from "@mui/icons-material/GroupsOutlined";
import LoginOutlinedIcon from "@mui/icons-material/LoginOutlined";
import LogoutOutlinedIcon from "@mui/icons-material/LogoutOutlined";
import RefreshOutlinedIcon from "@mui/icons-material/RefreshOutlined";
import { Alert, Box, Button, CircularProgress, List, ListItemButton, Tab, Tabs, TextField, Typography } from "@mui/material";
import { useEffect, useState, type ReactNode } from "react";
import { HubIconButton } from "../components/inputs";
import { hubTokens } from "../theme/tokens";
import { useAccount } from "./context";
import { accountCopy, type AccountCopy } from "./copy";
import { MemberAdministration } from "./MemberAdministration";
import { IssuedInvitations } from "./IssuedInvitations";
import { OperationNotice } from "./OperationNotice";
import type { Resource } from "./protocol";

export function AccountPanel({ language, selectedProjectId }: { language: string; selectedProjectId: string | null }) {
  const state = useAccount();
  const { controller } = state;
  const account = state.snapshot.account;
  const copy = accountCopy(language);
  const [tab, setTab] = useState<"members" | "projects">("members");
  const organization = state.organizations.items.find(item => item.id === state.selectedOrganization);
  const signedIn = account.status === "signed-in";
  const mutationBusy = state.mutation !== null || Boolean(state.snapshot.operationsError) || state.snapshot.operations.some(operation => operation.status === "unknown");
  const actor = state.members.items.find(member => member.active && member.issuer === account.issuer && member.subject === account.subject);

  useEffect(() => {
    if (signedIn) void controller.loadCloudBinding();
  }, [controller, signedIn, account.generation, selectedProjectId]);

  return (
    <Box component="section" aria-label={copy.title} data-testid="hub-account" sx={{ py: 2, mb: 2, minWidth: 0, borderBottom: `1px solid ${hubTokens.colors.line}` }}>
      <Box sx={{ display: "flex", alignItems: "center", flexWrap: "wrap", gap: 1, mb: 1.4 }}>
        <GroupsOutlinedIcon sx={{ color: hubTokens.colors.accent }} />
        <Typography variant="h6" sx={{ flex: "1 1 180px", fontSize: 18 }}>{copy.title}</Typography>
        {account.configured && state.ready ? (
          <HubIconButton label={copy.refresh} disabled={Boolean(state.pending) || state.mutation?.status === "running"} onClick={() => void controller.authenticate("refresh")} sx={iconSize}><RefreshOutlinedIcon /></HubIconButton>
        ) : state.ready && state.error ? <HubIconButton label={copy.retry} onClick={() => void controller.start()} sx={iconSize}><RefreshOutlinedIcon /></HubIconButton> : null}
        {signedIn ? <HubIconButton label={copy.signOut} onClick={() => void controller.authenticate("logout")} sx={iconSize}><LogoutOutlinedIcon /></HubIconButton> : null}
      </Box>
      {state.error ? <Alert severity="error" onClose={controller.clearError} sx={{ mb: 1.2 }}>{copy.error(state.error)}</Alert> : null}
      {!state.ready ? <Loading label={copy.loading} /> : state.pending ? (
        <Box sx={{ display: "flex", alignItems: "center", gap: 1, flexWrap: "wrap" }}>
          <Loading label={state.pending === "sign-in" ? copy.pending : copy.loading} />
          {state.pending !== "logout" && state.pending !== "cancel" ? <Button startIcon={<CloseOutlinedIcon />} onClick={() => void controller.authenticate("cancel")}>{copy.cancel}</Button> : null}
        </Box>
      ) : !account.configured ? <Typography variant="body2" color="text.secondary">{copy.unavailable}</Typography> : !signedIn ? (
        <Box sx={{ display: "flex", alignItems: "center", gap: 2 }}>
          <Typography variant="body2" color="text.secondary" sx={{ flex: 1 }}>{copy.signedOut}</Typography>
          <Button variant="contained" startIcon={<LoginOutlinedIcon />} onClick={() => void controller.authenticate("sign-in")}>{copy.signIn}</Button>
        </Box>
      ) : (
        <>
          <Typography variant="body2" data-testid="account-identity" sx={{ fontWeight: 700, overflowWrap: "anywhere" }}>{account.displayName || account.subject}</Typography>
          <Typography variant="caption" color="text.secondary" sx={{ display: "block", overflowWrap: "anywhere", mb: 2 }}>{account.issuer}</Typography>
          <OperationNotice copy={copy} />
          <Box sx={{ display: "grid", gridTemplateColumns: "minmax(200px, 1fr) minmax(0, 2fr)", gap: 2.5, "@media (max-width: 800px)": { gridTemplateColumns: "minmax(0, 1fr)" } }}>
            <Box data-testid="account-organizations" sx={{ minWidth: 0 }}>
              <ResourceSection title={copy.organizations} resource={state.organizations} copy={copy} onLoad={more => void controller.load("organizations", more)}>
                <List disablePadding dense>
                  {state.organizations.items.map(item => <ListItemButton key={item.id} selected={item.id === state.selectedOrganization} onClick={() => void controller.selectOrganization(item.id)} sx={{ gap: 1, px: 1, minWidth: 0 }}>
                    <Typography variant="body2" sx={{ flex: 1, minWidth: 0, overflowWrap: "anywhere" }}>{item.name}</Typography>
                    <CheckOutlinedIcon sx={{ fontSize: 18, visibility: item.id === state.selectedOrganization ? "visible" : "hidden" }} />
                  </ListItemButton>)}
                </List>
              </ResourceSection>
              <NameForm key={JSON.stringify([account.generation, account.issuer, account.subject, state.selectedOrganization])} label={copy.organizationName} action={copy.createOrganization} disabled={mutationBusy} onSubmit={controller.createOrganization} />
            </Box>
            <Box sx={{ minWidth: 0 }}>
              {organization ? (
                <>
                  <Typography variant="subtitle2" sx={{ overflowWrap: "anywhere" }}>{organization.name}</Typography>
                  <Tabs aria-label={copy.memberProjectTabs} value={tab} onChange={(_, value) => setTab(value)} sx={{ mb: 1, minHeight: 42 }}>
                    <Tab value="members" label={copy.members} sx={{ minHeight: 42 }} />
                    <Tab value="projects" label={copy.projects} sx={{ minHeight: 42 }} />
                  </Tabs>
                  {tab === "projects" ? <>
                    {!selectedProjectId ? <Typography variant="body2" color="text.secondary" sx={{ mb: 1 }}>{copy.selectLocalProjectToLink}</Typography> : null}
                    {state.cloudBindingLoading ? <Loading label={copy.loading} /> : null}
                    {state.cloudBindingError ? <Alert severity="error" sx={{ mb: 1 }}>{copy.error(state.cloudBindingError)}</Alert> : null}
                  </> : null}
                  {tab === "members" ? <><ResourceSection title={copy.members} resource={state.members} copy={copy} onLoad={more => void (more ? controller.load("members", true) : controller.refreshOrganization())}>
                    <MemberAdministration key={JSON.stringify([account.generation, account.issuer, account.subject, organization.id])} organization={organization} copy={copy} disabled={mutationBusy} />
                  </ResourceSection><IssuedInvitations key={JSON.stringify([account.generation, account.issuer, account.subject, organization.id])} organization={organization} copy={copy} disabled={mutationBusy} /></> : <>
                    <ResourceSection title={copy.projects} resource={state.projects} copy={copy} onLoad={more => void controller.load("projects", more)}>
                      {state.projects.items.map(project => {
                        const linked = state.cloudBinding?.organizationId === organization.id && state.cloudBinding.projectId === project.id;
                        return <Box key={project.id} sx={{ ...recordSx, display: "flex", alignItems: "center", flexWrap: "wrap", gap: 1 }}>
                          <Typography variant="body2" sx={{ flex: "1 1 180px", minWidth: 0, overflowWrap: "anywhere" }}>{project.name}</Typography>
                          {linked ? <Typography variant="caption" color="text.secondary">{copy.linkedLocalProject}</Typography> : <Button size="small" disabled={!selectedProjectId || mutationBusy || state.cloudBindingLoading || state.projects.loading} onClick={() => { if (selectedProjectId) void controller.attachCloudProject(project.id, selectedProjectId); }}>{copy.linkSelectedProject}</Button>}
                        </Box>;
                      })}
                    </ResourceSection>
                    {actor && actor.role !== "viewer" ? <NameForm key={JSON.stringify([account.generation, account.issuer, account.subject, organization.id])} label={copy.projectName} action={copy.createProject} disabled={mutationBusy || state.organizationsNeedingRefresh.includes(organization.id) || state.organizations.loading || state.members.loading || Boolean(state.members.error)} onSubmit={name => controller.mutate(organization.id, organization.policyRevision, { action: "create-project", name })} /> : null}
                  </>}
                </>
              ) : <Typography variant="body2" color="text.secondary">{copy.select}</Typography>}
            </Box>
          </Box>
          <Box sx={{ mt: 2.5 }}>
            <ResourceSection title={copy.invitations} resource={state.invitations} copy={copy} onLoad={more => void controller.load("invitations", more)}>
              {state.invitations.items.some(item => state.organizationsNeedingRefresh.includes(item.organizationId)) ? <Alert severity="warning" sx={{ my: 1 }}>{copy.refreshInvitations}</Alert> : null}
              {state.invitations.items.map(invitation => {
                const expired = invitation.expiresAt * 1000 <= Date.now();
                return <Box key={invitation.id} sx={{ ...recordSx, display: "flex", flexWrap: "wrap", alignItems: "center", gap: 1 }}>
                  <Box sx={{ flex: "1 1 180px", minWidth: 0 }}>
                    <Typography variant="body2" sx={{ overflowWrap: "anywhere" }}>{invitation.organizationName}</Typography>
                    <Typography variant="caption" color="text.secondary">{copy.roles[invitation.role]} · {invitation.status === "pending" && expired ? copy.expired : copy.invitationStates[invitation.status]}</Typography>
                  </Box>
                  {invitation.status === "pending" && !expired ? <Button size="small" startIcon={<CheckOutlinedIcon />} disabled={mutationBusy || state.organizationsNeedingRefresh.includes(invitation.organizationId)} onClick={() => void controller.mutate(invitation.organizationId, invitation.policyRevision, { action: "accept-invite", invitation_id: invitation.id })}>{copy.accept}</Button> : null}
                </Box>;
              })}
            </ResourceSection>
          </Box>
        </>
      )}
    </Box>
  );
}

function ResourceSection<T>({ title, resource, copy, onLoad, children }: { title: string; resource: Resource<T>; copy: AccountCopy; onLoad: (more: boolean) => void; children: ReactNode }) {
  return <Box sx={{ minWidth: 0 }}>
    <Box sx={{ display: "flex", alignItems: "center", gap: 1, mb: 0.5 }}>
      <Typography variant="subtitle2" sx={{ flex: 1 }}>{title}</Typography>
      <HubIconButton label={`${copy.refreshList}: ${title}`} disabled={resource.loading} onClick={() => onLoad(false)} sx={iconSize}><RefreshOutlinedIcon fontSize="small" /></HubIconButton>
    </Box>
    {resource.error ? <Alert severity="error" sx={{ mb: 1 }}>{copy.error(resource.error)}</Alert> : null}
    {children}
    {resource.loading ? <Loading label={copy.loading} /> : resource.loaded && resource.items.length === 0 ? <Typography variant="body2" color="text.secondary">{copy.none}</Typography> : null}
    {resource.nextCursor ? <Button onClick={() => onLoad(true)} disabled={resource.loading} size="small">{copy.more}</Button> : null}
  </Box>;
}

function NameForm({ label, action, disabled, onSubmit }: { label: string; action: string; disabled: boolean; onSubmit: (value: string) => Promise<boolean> }) {
  const [name, setName] = useState("");
  const valid = name.trim().length > 0 && new TextEncoder().encode(name.trim()).length <= 256 && !/[\x00-\x1f\x7f]/.test(name);
  return <Box component="form" onSubmit={event => { event.preventDefault(); if (!disabled && valid) void onSubmit(name.trim()).then(success => { if (success) setName(""); }); }} sx={{ display: "flex", alignItems: "center", gap: 0.8, mt: 1.5 }}>
    <TextField label={label} size="small" value={name} onChange={event => setName(event.target.value)} disabled={disabled} slotProps={{ htmlInput: { maxLength: 256 } }} sx={{ flex: 1, minWidth: 0 }} />
    <HubIconButton type="submit" label={action} disabled={disabled || !valid} sx={iconSize}><AddOutlinedIcon /></HubIconButton>
  </Box>;
}

function Loading({ label }: { label: string }) { return <Box role="status" sx={{ display: "flex", alignItems: "center", gap: 1, py: 0.5 }}><CircularProgress size={16} /><Typography variant="body2">{label}</Typography></Box>; }
const iconSize = { width: 34, height: 34, flexShrink: 0 };
const recordSx = { display: "grid", gap: 0.2, py: 0.75, borderBottom: `1px solid ${hubTokens.colors.line}`, overflowWrap: "anywhere" };
