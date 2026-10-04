import { Component, type ErrorInfo, type ReactNode } from "react";
import { Alert, Box, Typography } from "@mui/material";
import { HubButton } from "../inputs";
import type { HubShellText } from "../../types/hub";

// 壳层提供已就绪快照的文案与恢复入口；恢复回调须重新获取可渲染状态，当前应用会先切回启动界面。
interface HubErrorBoundaryProps {
  children: ReactNode;
  shellText: HubShellText;
  onReset: () => void;
}

// 渲染失败锁存在边界实例内，避免后续普通更新再次进入失效子树；重新加载期间实例会被壳层卸载。
interface HubErrorBoundaryState {
  error: Error | null;
}

// 隔离已就绪界面的渲染异常；异步命令失败仍由动作链处理，此边界不接管命令回执。
export class HubErrorBoundary extends Component<HubErrorBoundaryProps, HubErrorBoundaryState> {
  state: HubErrorBoundaryState = { error: null };

  // 由渲染框架进入失败展示，不能在这个状态派生阶段发起恢复请求。
  static getDerivedStateFromError(error: Error): HubErrorBoundaryState {
    return { error };
  }

  // 保留异常及组件栈供诊断；用户可见恢复说明仍来自当前壳层文案。
  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    console.error(this.props.shellText.actionFailed, error, errorInfo);
  }

  // 子树失败时由边界提供恢复入口；恢复需要壳层重新装载权威快照，不只是清除这次异常。
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
