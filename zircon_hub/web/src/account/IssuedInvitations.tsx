import BlockOutlinedIcon from "@mui/icons-material/BlockOutlined";
import CloseOutlinedIcon from "@mui/icons-material/CloseOutlined";
import RefreshOutlinedIcon from "@mui/icons-material/RefreshOutlined";
import { Alert, Box, Button, CircularProgress, Typography } from "@mui/material";
import { useEffect, useRef, useState } from "react";
import { HubIconButton } from "../components/inputs";
import { HubDialog } from "../components/overlays/HubDialog";
import { hubTokens } from "../theme/tokens";
import { useAccount } from "./context";
import type { AccountCopy } from "./copy";
import type { IssuedInvitation, Organization } from "./protocol";

export function IssuedInvitations({ organization, copy, disabled }: { organization: Organization; copy: AccountCopy; disabled: boolean }) {
  const state = useAccount();
  const { controller, issuedInvitations: resource, snapshot: { account } } = state;
  const [draft, setDraft] = useState<IssuedInvitation | null>(null);
  const [refreshing, setRefreshing] = useState(false);
  const [reconfirmed, setReconfirmed] = useState(false);
  const attempt = useRef(0);
  const identity = JSON.stringify([account.generation, account.issuer, account.subject, organization.id]);
  useEffect(() => { attempt.current++; setDraft(null); setRefreshing(false); setReconfirmed(false); return () => { attempt.current++; }; }, [identity]);
  const actor = state.members.items.find(member => member.issuer === account.issuer && member.subject === account.subject);
  const allowed = state.members.loaded && !state.members.loading && !state.members.error && actor?.active && (actor.role === "owner" || actor.role === "admin");
  const current = draft ? resource.items.find(item => item.id === draft.id) : undefined;
  const needsRefresh = state.organizationsNeedingRefresh.includes(organization.id);
  const policyChanged = current !== undefined && draft !== null && current.policyRevision !== draft.policyRevision;
  const locked = disabled || refreshing || state.organizations.loading || resource.loading || Boolean(resource.error);
  const eligible = current?.status === "pending" && current.expiresAt * 1000 > Date.now();
  const close = () => { attempt.current++; setDraft(null); setRefreshing(false); setReconfirmed(false); };
  const refresh = async () => {
    if (!draft || refreshing) return;
    const ticket = ++attempt.current;
    setRefreshing(true);
    const refreshed = await controller.refreshOrganization(undefined, draft.id);
    if (ticket !== attempt.current) return;
    setRefreshing(false);
    if (refreshed) {
      const next = controller.getSnapshot().issuedInvitations.items.find(item => item.id === draft.id);
      if (next) { setDraft(next); setReconfirmed(true); }
    }
  };
  const revoke = async () => {
    if (!draft || !allowed || locked || !eligible || needsRefresh || policyChanged || draft.expiresAt * 1000 <= Date.now()) return;
    const ticket = attempt.current;
    const success = await controller.mutate(organization.id, draft.policyRevision, { action: "revoke-invite", invitation_id: draft.id });
    if (ticket !== attempt.current) return;
    const next = controller.getSnapshot();
    if (success || next.mutation?.status === "unknown" || next.snapshot.operationsError || next.snapshot.operations.some(item => item.status === "unknown")) close();
  };

  return <>
    {allowed ? <Box data-testid="account-issued-invitations" sx={{ mt: 2.5, minWidth: 0 }}>
      <Box sx={{ display: "flex", alignItems: "center", gap: 1, mb: 0.5 }}>
        <Typography variant="subtitle2" sx={{ flex: 1, minWidth: 0 }}>{copy.issuedInvitations}</Typography>
        <HubIconButton label={`${copy.refreshList}: ${copy.issuedInvitations}`} disabled={resource.loading || state.organizations.loading || state.mutation?.status === "running"} onClick={() => void controller.refreshOrganization()} sx={{ width: 34, height: 34 }}><RefreshOutlinedIcon fontSize="small" /></HubIconButton>
      </Box>
      {resource.error ? <Alert severity="error" sx={{ my: 1 }}>{copy.error(resource.error)}</Alert> : null}
      {resource.items.map(invitation => {
        const expired = invitation.status === "expired" || (invitation.status === "pending" && invitation.expiresAt * 1000 <= Date.now());
        return <Box key={invitation.id} data-testid={`account-issued-${invitation.id}`} sx={{ display: "flex", flexWrap: "wrap", alignItems: "center", gap: 1, py: 0.8, borderBottom: `1px solid ${hubTokens.colors.line}`, minWidth: 0 }}>
          <Box sx={{ flex: "1 1 170px", display: "grid", gap: 0.2, minWidth: 0 }}>
            <Typography variant="body2" sx={wrap}>{invitation.targetSubject}</Typography>
            <Typography variant="caption" color="text.secondary" sx={wrap}>{invitation.targetIssuer}</Typography>
            <Typography variant="caption" color="text.secondary" sx={wrap}>{copy.roles[invitation.role]} · {expired ? copy.expired : copy.invitationStates[invitation.status]}</Typography>
            <Typography variant="caption" color="text.secondary" sx={wrap}>{copy.expiresAt}: {new Date(invitation.expiresAt * 1000).toLocaleString(copy.pair("en-US", "zh-CN"))}</Typography>
          </Box>
          {invitation.status === "pending" && !expired ? <Button size="small" startIcon={<BlockOutlinedIcon />} disabled={locked} sx={{ maxWidth: "100%", whiteSpace: "normal" }} onClick={() => { attempt.current++; setReconfirmed(false); setDraft(invitation); }}>{copy.revokeInvitation}</Button> : null}
        </Box>;
      })}
      {resource.loading ? <Box role="status" sx={{ display: "flex", alignItems: "center", gap: 1, py: 1 }}><CircularProgress size={16} /><Typography variant="body2">{copy.loading}</Typography></Box> : resource.loaded && resource.items.length === 0 ? <Typography variant="body2" color="text.secondary">{copy.none}</Typography> : null}
      {resource.nextCursor ? <Button size="small" disabled={resource.loading} onClick={() => void controller.load("issued-invitations", true)}>{copy.more}</Button> : null}
    </Box> : null}
    <HubDialog open={draft !== null} title={copy.revokeInvitation} onClose={close} actions={<Box sx={{ display: "flex", flexWrap: "wrap", justifyContent: "flex-end", gap: 1, p: 1, width: "100%" }}>
      <Button startIcon={<CloseOutlinedIcon />} onClick={close}>{copy.closeDialog}</Button>
      {needsRefresh || policyChanged ? <Button startIcon={<RefreshOutlinedIcon />} disabled={refreshing || state.mutation?.status === "running"} onClick={() => void refresh()}>{copy.refreshOrganization}</Button>
        : <Button variant="contained" color="warning" startIcon={<BlockOutlinedIcon />} disabled={locked || !allowed || !eligible} onClick={() => void revoke()}>{copy.confirmRevoke}</Button>}
    </Box>}>
      {draft ? <Box sx={{ display: "grid", gap: 1.4, pt: 0.5, minWidth: 0 }}>
        <Typography variant="subtitle2" sx={wrap}>{organization.name}</Typography>
        <Box sx={{ display: "grid", gap: 0.3, minWidth: 0 }}><Typography variant="body2" sx={wrap}>{draft.targetSubject}</Typography><Typography variant="caption" color="text.secondary" sx={wrap}>{draft.targetIssuer}</Typography></Box>
        {needsRefresh || policyChanged ? <Alert severity="warning">{needsRefresh ? copy.refreshRequired : copy.policyChanged}</Alert> : reconfirmed ? <Alert severity="info">{copy.confirmAgain}</Alert> : null}
        {!refreshing && resource.loaded && (!eligible || !allowed) ? <Alert severity="warning">{allowed ? copy.invitationUnavailable : copy.permissionUnavailable}</Alert> : <Alert severity="warning">{copy.revokeWarning}</Alert>}
      </Box> : null}
    </HubDialog>
  </>;
}

const wrap = { overflowWrap: "anywhere", minWidth: 0 } as const;
