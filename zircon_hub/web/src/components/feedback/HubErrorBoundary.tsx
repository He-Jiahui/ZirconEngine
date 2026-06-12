import { Component, type ErrorInfo, type ReactNode } from "react";
import { Alert, Box, Typography } from "@mui/material";
import { HubButton } from "../inputs";
import type { HubShellText } from "../../types/hub";

interface HubErrorBoundaryProps {
  children: ReactNode;
  shellText: HubShellText;
  onReset: () => void;
}

interface HubErrorBoundaryState {
  error: Error | null;
}

export class HubErrorBoundary extends Component<HubErrorBoundaryProps, HubErrorBoundaryState> {
  state: HubErrorBoundaryState = { error: null };

  static getDerivedStateFromError(error: Error): HubErrorBoundaryState {
    return { error };
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    console.error(this.props.shellText.actionFailed, error, errorInfo);
  }

  render() {
    const { children, shellText } = this.props;
    const { error } = this.state;

    if (!error) {
      return children;
    }

    return (
      <Box sx={{ width: "100vw", height: "100vh", display: "grid", placeItems: "center", p: 4 }}>
        <Alert severity="error" variant="outlined" sx={{ width: "min(560px, 100%)" }}>
          <Box sx={{ display: "grid", gap: 1.2 }}>
            <Typography variant="subtitle1">{shellText.actionFailed}</Typography>
            <Typography variant="body2">{shellText.actionFailedDetail}</Typography>
            <Typography variant="caption" color="text.secondary">
              {shellText.checkActionTarget}
            </Typography>
            <Box>
              <HubButton
                onClick={() => {
                  this.setState({ error: null });
                  this.props.onReset();
                }}
              >
                {shellText.stateRefreshAfterCommand}
              </HubButton>
            </Box>
          </Box>
        </Alert>
      </Box>
    );
  }
}
