import CheckOutlinedIcon from "@mui/icons-material/CheckOutlined";
import CloseOutlinedIcon from "@mui/icons-material/CloseOutlined";
import DownloadOutlinedIcon from "@mui/icons-material/DownloadOutlined";
import ExtensionOutlinedIcon from "@mui/icons-material/ExtensionOutlined";
import Inventory2OutlinedIcon from "@mui/icons-material/Inventory2Outlined";
import LoginOutlinedIcon from "@mui/icons-material/LoginOutlined";
import LogoutOutlinedIcon from "@mui/icons-material/LogoutOutlined";
import PolicyOutlinedIcon from "@mui/icons-material/PolicyOutlined";
import RefreshOutlinedIcon from "@mui/icons-material/RefreshOutlined";
import SearchOutlinedIcon from "@mui/icons-material/SearchOutlined";
import { Alert, Box, Button, Checkbox, CircularProgress, FormControlLabel, List, ListItemButton, MenuItem, TextField, Typography } from "@mui/material";
import { useEffect, useRef, useState } from "react";
import { useAccount } from "../../account/context";
import { accountCopy } from "../../account/copy";
import { OperationNotice } from "../../account/OperationNotice";
import { HubIconButton } from "../../components/inputs";
import { HubDialog } from "../../components/overlays/HubDialog";
import { StatusBadge } from "../../components/data";
import { hubTokens } from "../../theme/tokens";
import { catalogCopy } from "./copy";
import { isInstalled, supportsPackageInstall, type CatalogRelease, type PackageRuntimeMode } from "./protocol";

export function ServiceCatalog({ mode, language }: { mode: "assets" | "plugins"; language: string }) {
  const state = useAccount();
  const { controller, serviceCatalog: catalog, catalogEntitlements: entitlements, packageInventory: inventory } = state;
  const account = state.snapshot.account;
  const copy = catalogCopy(language);
  const common = accountCopy(language);
  const [query, setQuery] = useState(state.catalogQuery);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [targetMode, setTargetMode] = useState<PackageRuntimeMode>("editor_host");
  const signedIn = account.status === "signed-in";
  const releases = catalog.items.filter(item => item.kind === (mode === "plugins" ? "plugin" : "asset"));
  const selected = releases.find(item => item.package_id === selectedId) ?? releases[0];
  const organization = state.organizations.items.find(item => item.id === state.selectedOrganization);
  const actor = state.members.items.find(item => item.active && item.issuer === account.issuer && item.subject === account.subject);
  const accessReady = Boolean(organization && !state.organizations.loading && state.members.loaded && !state.members.loading && !state.members.error && actor);
  const administrator = accessReady && (actor?.role === "owner" || actor?.role === "admin");
  const busy = state.mutation !== null || Boolean(state.snapshot.operationsError) || state.snapshot.operations.some(item => item.status === "unknown");
  const needsRefresh = Boolean(organization && state.organizationsNeedingRefresh.includes(organization.id));
  const licensed = Boolean(selected && entitlements.loaded && !entitlements.error && entitlements.items.some(item => item.packageId === selected.package_id && item.revision === selected.revision && item.licenseId === selected.license_id));
  const licenseReady = entitlements.loaded && !entitlements.loading && !entitlements.error && (licensed || entitlements.nextCursor === null);
  const installationPending = state.mutation?.status === "unknown" || state.snapshot.operations.some(item => item.action === "package-install" && item.organizationId === organization?.id && item.status === "unknown");
  const inventoryReady = inventory.data?.targetMode === targetMode && !inventory.loading && !inventory.error && !installationPending;
  const installed = Boolean(selected && inventoryReady && isInstalled(selected, inventory.data, targetMode));
  const expired = Boolean(selected && selected.exp * 1000 <= Date.now());
  const locked = busy || !accessReady || needsRefresh || catalog.loading || Boolean(catalog.error) || expired;
  const queryValid = new TextEncoder().encode(query).length <= 256;
  const identity = JSON.stringify([account.generation, account.issuer, account.subject, state.selectedOrganization]);

  // The controller clears its authoritative query at account boundaries. Keep
  // the input draft in sync with that reset while preserving unsubmitted text
  // during ordinary renders.
  useEffect(() => { setQuery(state.catalogQuery); }, [identity, state.catalogQuery]);
  useEffect(() => { if (state.ready && signedIn && !catalog.loaded && !catalog.loading && !catalog.error) void controller.loadCatalog(); }, [controller, state.ready, signedIn, account.generation, catalog.loaded, catalog.loading, catalog.error]);
  useEffect(() => { if (accessReady && !entitlements.loaded && !entitlements.loading && !entitlements.error) void controller.loadCatalogEntitlements(); }, [controller, accessReady, identity, entitlements.loaded, entitlements.loading, entitlements.error]);
  useEffect(() => { if (mode === "plugins" && accessReady && inventory.data?.targetMode !== targetMode && inventory.targetMode !== targetMode && !inventory.loading) void controller.loadPackageInventory(targetMode); }, [controller, mode, accessReady, identity, targetMode, inventory.data?.targetMode, inventory.targetMode, inventory.loading]);

  return <Box component="section" aria-label={copy.service} data-testid="service-catalog" sx={{ minWidth: 0, pt: 2 }}>
    {state.error ? <Alert severity="error" onClose={controller.clearError} sx={{ mb: 1.5 }}>{common.error(state.error)}</Alert> : null}
    {!state.ready || state.pending ? <Box sx={toolbar}><Loading label={state.pending === "sign-in" ? common.pending : common.loading} />{state.pending && state.pending !== "logout" && state.pending !== "cancel" ? <Button startIcon={<CloseOutlinedIcon />} onClick={() => void controller.authenticate("cancel")}>{common.cancel}</Button> : null}</Box>
      : !signedIn ? <Box sx={toolbar}><Typography variant="body2" color="text.secondary">{account.configured ? common.signedOut : common.unavailable}</Typography>{account.configured ? <Button startIcon={<LoginOutlinedIcon />} onClick={() => void controller.authenticate("sign-in")}>{common.signIn}</Button> : state.error ? <Button startIcon={<RefreshOutlinedIcon />} onClick={() => void controller.start()}>{common.retry}</Button> : null}</Box>
      : <>
        <Box sx={{ ...toolbar, mb: 1.5 }}>
          <TextField select size="small" label={copy.organization} value={organization?.id ?? ""} onChange={event => void controller.selectOrganization(event.target.value)} disabled={state.organizations.loading || state.mutation?.status === "running"} sx={{ flex: "1 1 220px", minWidth: 0, maxWidth: 440, "&& .MuiSelect-select": { whiteSpace: "normal", overflowWrap: "anywhere", textOverflow: "clip", height: "auto" } }}>
            <MenuItem value="" disabled>{copy.chooseOrganization}</MenuItem>
            {state.organizations.items.map(item => <MenuItem key={item.id} value={item.id} sx={{ whiteSpace: "normal", overflowWrap: "anywhere", maxWidth: "min(440px, calc(100vw - 32px))" }}>{item.name}</MenuItem>)}
          </TextField>
          <HubIconButton label={organization ? common.refreshOrganization : `${common.refreshList}: ${common.organizations}`} disabled={state.organizations.loading || state.mutation?.status === "running"} sx={iconSize} onClick={() => void (organization ? controller.refreshOrganization() : controller.load("organizations"))}><RefreshOutlinedIcon /></HubIconButton>
          {mode === "plugins" ? <TextField select size="small" label={copy.installTarget} value={targetMode} onChange={event => setTargetMode(event.target.value as PackageRuntimeMode)} disabled={inventory.loading || state.mutation?.status === "running"} sx={{ flex: "1 1 200px", minWidth: 0, maxWidth: 320, "&& .MuiSelect-select": { whiteSpace: "normal", overflowWrap: "anywhere", textOverflow: "clip", height: "auto" } }}>
            <MenuItem value="editor_host">{copy.editorHostTarget}</MenuItem>
            <MenuItem value="client_runtime">{copy.clientRuntimeTarget}</MenuItem>
          </TextField> : null}
          {state.organizations.nextCursor ? <Button size="small" disabled={state.organizations.loading} onClick={() => void controller.load("organizations", true)}>{common.more}</Button> : null}
          <Typography variant="caption" sx={{ ...wrap, flex: "1 1 150px" }} color="text.secondary">{account.displayName ?? account.subject}{accessReady && actor ? ` / ${common.roles[actor.role]}` : ""}</Typography>
          <HubIconButton label={common.signOut} sx={iconSize} onClick={() => void controller.authenticate("logout")}><LogoutOutlinedIcon /></HubIconButton>
        </Box>
        {state.organizations.error ? <Alert severity="error" sx={{ mb: 1 }}>{common.error(state.organizations.error)}</Alert> : null}
        {state.members.error ? <Alert severity="error" sx={{ mb: 1 }}>{common.error(state.members.error)}</Alert> : null}
        <OperationNotice copy={common} />
        {needsRefresh ? <Alert severity="warning" sx={{ mb: 1.5 }}>{common.refreshRequired}</Alert> : null}
        <Box component="form" onSubmit={event => { event.preventDefault(); if (queryValid) void controller.loadCatalog(query); }} sx={{ ...toolbar, mb: 1.5 }}>
          <TextField size="small" label={copy.search} value={query} onChange={event => setQuery(event.target.value)} error={!queryValid} helperText={!queryValid ? copy.searchTooLong : undefined} sx={{ flex: "1 1 220px", minWidth: 0 }} />
          <HubIconButton type="submit" label={copy.searchAction} disabled={!queryValid} sx={iconSize}><SearchOutlinedIcon /></HubIconButton>
          <HubIconButton label={`${common.refreshList}: ${copy.service}`} disabled={catalog.loading} sx={iconSize} onClick={() => void controller.loadCatalog()}><RefreshOutlinedIcon /></HubIconButton>
        </Box>
        {catalog.error ? <Alert severity="error" sx={{ mb: 1.5 }}>{common.error(catalog.error)}</Alert> : null}
        <Box sx={{ display: "grid", gridTemplateColumns: "minmax(0, 1fr) minmax(300px, 0.8fr)", gap: 3, alignItems: "start", "@media (max-width: 980px)": { gridTemplateColumns: "minmax(0, 1fr)" } }}>
          <Box sx={{ minWidth: 0 }}>
            <List disablePadding aria-label={copy.service}>
              {releases.map(release => <ListItemButton key={release.package_id} selected={release === selected} onClick={() => setSelectedId(release.package_id)} sx={{ gap: 1.2, py: 1.4, px: 1, minWidth: 0, borderBottom: `1px solid ${hubTokens.colors.line}`, alignItems: "start" }}>
                {release.kind === "plugin" ? <ExtensionOutlinedIcon sx={entryIcon} /> : <Inventory2OutlinedIcon sx={entryIcon} />}
                <Box sx={{ minWidth: 0, flex: 1 }}>
                  <Typography variant="body2" sx={{ ...wrap, fontWeight: 700 }}>{release.name}</Typography>
                  <Typography variant="caption" color="text.secondary" sx={{ ...wrap, display: "block" }}>{release.version}</Typography>
                  <Typography variant="body2" color="text.secondary" sx={wrap}>{release.description}</Typography>
                </Box>
                {inventoryReady && isInstalled(release, inventory.data, targetMode) ? <CheckOutlinedIcon aria-label={copy.installed} sx={{ color: hubTokens.colors.success, fontSize: 19, flexShrink: 0 }} /> : null}
              </ListItemButton>)}
            </List>
            {catalog.loading ? <Loading label={common.loading} /> : catalog.loaded && releases.length === 0 ? <Typography variant="body2" color="text.secondary" sx={{ py: 2 }}>{copy.noReleases}</Typography> : null}
            {catalog.nextCursor ? <Button disabled={catalog.loading} sx={{ mt: 1 }} onClick={() => void controller.loadCatalog(undefined, true)}>{common.more}</Button> : null}
          </Box>
          {selected ? <Box data-testid="service-release" sx={{ minWidth: 0, pt: 1 }}>
            <Typography variant="h6" sx={{ ...wrap, fontSize: 18, mb: 0.75 }}>{selected.name}</Typography>
            <Box sx={{ ...toolbar, mb: 1.5 }}>{selected.kind === "plugin" ? <StatusBadge label={expired ? common.expired : !inventoryReady ? copy.inventoryPending : installed ? copy.installed : copy.notInstalled} tone={installed ? "success" : "neutral"} /> : expired ? <StatusBadge label={common.expired} tone="neutral" /> : null}<StatusBadge label={!licenseReady ? copy.licensePending : licensed ? copy.accepted : copy.licenseRequired} tone={licensed ? "success" : "neutral"} /></Box>
            <dl style={{ margin: 0 }}>
              <Detail label={copy.version} value={`${selected.version} / ${selected.revision}`} />
              <Detail label={copy.publisher} value={selected.sub} />
              <Detail label={common.identityIssuer} value={selected.iss} />
              <Detail label={copy.size} value={`${new Intl.NumberFormat(language === "Chinese" ? "zh-CN" : "en").format(selected.artifact_size)} B`} />
              <Detail label={copy.license} value={selected.license_id} />
            </dl>
            {!organization ? <Typography variant="body2" color="text.secondary" sx={{ mt: 1.5 }}>{copy.chooseOrganization}</Typography> : <>
              {entitlements.loading || inventory.loading || state.members.loading ? <Loading label={common.loading} /> : null}
              {entitlements.error ? <Alert severity="error" sx={{ mt: 1 }}>{common.error(entitlements.error)}<Button size="small" startIcon={<RefreshOutlinedIcon />} onClick={() => void controller.loadCatalogEntitlements()}>{common.retry}</Button></Alert> : null}
              {inventory.error && inventory.targetMode === targetMode ? <Alert severity="error" sx={{ mt: 1 }}>{common.error(inventory.error)}<Button size="small" startIcon={<RefreshOutlinedIcon />} onClick={() => void controller.loadPackageInventory(targetMode)}>{common.retry}</Button></Alert> : null}
              {entitlements.nextCursor ? <Button size="small" disabled={entitlements.loading} onClick={() => void controller.loadCatalogEntitlements(true)}>{copy.licenseMore}</Button> : null}
              {!licensed && licenseReady && accessReady && !administrator ? <Typography variant="body2" color="text.secondary" sx={{ mt: 1 }}>{copy.adminRequired}</Typography> : null}
            </>}
            <Box sx={{ ...toolbar, mt: 1.5 }}>
              <LicenseAction key={JSON.stringify([identity, selected.package_id, selected.revision])} release={selected} language={language} disabled={locked || !administrator || !licenseReady || licensed} />
              {selected.kind === "plugin" ? <Button variant="contained" startIcon={<DownloadOutlinedIcon />} disabled={locked || !supportsPackageInstall(selected) || !licensed || !licenseReady || !inventoryReady || installed} onClick={() => void controller.installCatalogPackage(selected, targetMode)}>{copy.install}</Button> : null}
            </Box>
            {selected.kind === "plugin" && !supportsPackageInstall(selected) ? <Typography variant="body2" color="text.secondary" sx={{ mt: 1 }}>{copy.packageTooLarge}</Typography> : null}
          </Box> : null}
        </Box>
        {organization ? <Box component="section" aria-label={copy.inventory} sx={{ mt: 3, pt: 2, borderTop: `1px solid ${hubTokens.colors.line}`, minWidth: 0 }}>
          <Box sx={toolbar}><Typography variant="subtitle2" sx={{ flex: 1 }}>{copy.inventory} / {targetMode === "editor_host" ? copy.editorHostTarget : copy.clientRuntimeTarget}</Typography><HubIconButton label={`${common.refreshList}: ${copy.inventory}`} sx={iconSize} disabled={inventory.loading || !accessReady} onClick={() => void controller.loadPackageInventory(targetMode)}><RefreshOutlinedIcon /></HubIconButton></Box>
          {inventoryReady && inventory.data ? <>
            <Typography variant="caption" color="text.secondary">{copy.inventoryRevision}: {inventory.data.revision}</Typography>
            {inventory.data.packages.length === 0 ? <Typography variant="body2" color="text.secondary" sx={{ mt: 1 }}>{copy.inventoryEmpty}</Typography> : inventory.data.packages.map(item => <Box key={item.packageId} sx={{ py: 1, borderBottom: `1px solid ${hubTokens.colors.line}`, minWidth: 0 }}>
              <Typography variant="body2" sx={wrap}>{catalog.items.find(release => release.package_id === item.packageId)?.name ?? item.packageId} / {item.version}</Typography>
              <Typography variant="caption" color="text.secondary" sx={{ ...wrap, display: "block" }}>{copy.packageId}: {item.packageId}</Typography>
              <Typography variant="caption" color="text.secondary" sx={{ ...wrap, display: "block" }}>{copy.digest}: {item.artifactDigest}</Typography>
            </Box>)}
          </> : <Typography variant="body2" color="text.secondary">{copy.inventoryPending}</Typography>}
        </Box> : null}
      </>}
  </Box>;
}

function LicenseAction({ release, language, disabled }: { release: CatalogRelease; language: string; disabled: boolean }) {
  const state = useAccount();
  const { controller } = state;
  const copy = catalogCopy(language);
  const common = accountCopy(language);
  const organization = state.organizations.items.find(item => item.id === state.selectedOrganization);
  const [revision, setRevision] = useState<string | null>(null);
  const [open, setOpen] = useState(false);
  const [accepted, setAccepted] = useState(false);
  const attempt = useRef(0);
  useEffect(() => () => { attempt.current++; }, []);
  const changed = revision !== null && revision !== organization?.policyRevision;
  const close = () => { attempt.current++; setOpen(false); setRevision(null); setAccepted(false); };
  const submit = async () => {
    if (disabled || changed || !accepted) return;
    const ticket = attempt.current;
    const success = await controller.acceptCatalogLicense(release);
    if (ticket !== attempt.current) return;
    const current = controller.getSnapshot();
    if (success || current.mutation?.status === "unknown" || current.snapshot.operationsError || current.snapshot.operations.some(item => item.status === "unknown")) close();
  };
  return <>
    <Button startIcon={<PolicyOutlinedIcon />} onClick={() => { setOpen(true); setRevision(organization?.policyRevision ?? null); setAccepted(false); }}>{copy.reviewLicense}</Button>
    <HubDialog open={open} title={copy.license} onClose={close} actions={<Box sx={{ ...toolbar, justifyContent: "flex-end", p: 1, width: "100%" }}>
      <Button startIcon={<CloseOutlinedIcon />} onClick={close}>{common.closeDialog}</Button>
      <Button variant="contained" startIcon={<CheckOutlinedIcon />} disabled={disabled || changed || !accepted} onClick={() => void submit()}>{copy.acceptLicense}</Button>
    </Box>}>
      <Box sx={{ display: "grid", gap: 1.5, minWidth: 0 }}>
        <Typography variant="subtitle2" sx={wrap}>{organization?.name}</Typography>
        <Typography variant="body2" sx={wrap}>{release.name} / {release.version} / {release.license_id}</Typography>
        {changed ? <Alert severity="warning">{common.policyChanged}</Alert> : null}
        <Typography component="pre" variant="body2" sx={{ ...wrap, whiteSpace: "pre-wrap", m: 0, maxHeight: 280, overflow: "auto", p: 1.5, border: `1px solid ${hubTokens.colors.line}` }}>{release.license_text}</Typography>
        <FormControlLabel control={<Checkbox checked={accepted} disabled={disabled || changed} onChange={event => setAccepted(event.target.checked)} />} label={copy.licenseConsent} sx={{ alignItems: "start", "& .MuiFormControlLabel-label": { overflowWrap: "anywhere", minWidth: 0, pt: 1 } }} />
      </Box>
    </HubDialog>
  </>;
}

function Detail({ label, value }: { label: string; value: string }) { return <Box sx={{ display: "grid", gridTemplateColumns: "minmax(85px, 0.45fr) minmax(0, 1fr)", gap: 1, py: 0.8, borderBottom: `1px solid ${hubTokens.colors.line}` }}><Typography component="dt" variant="caption" color="text.secondary" sx={wrap}>{label}</Typography><Typography component="dd" variant="body2" sx={{ ...wrap, m: 0 }}>{value}</Typography></Box>; }
function Loading({ label }: { label: string }) { return <Box role="status" sx={{ ...toolbar, py: 1 }}><CircularProgress size={16} /><Typography variant="body2">{label}</Typography></Box>; }
const toolbar = { display: "flex", flexWrap: "wrap", alignItems: "center", gap: 1, minWidth: 0 } as const;
const wrap = { overflowWrap: "anywhere", minWidth: 0 } as const;
const iconSize = { width: 36, height: 36, flexShrink: 0 };
const entryIcon = { color: hubTokens.colors.textMuted, fontSize: 24, flexShrink: 0, mt: 0.3 };
