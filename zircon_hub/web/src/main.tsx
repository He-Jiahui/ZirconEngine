import React from "react";
import ReactDOM from "react-dom/client";
import { CssBaseline, ThemeProvider } from "@mui/material";
import { App } from "./App";
import { hubTheme } from "./theme/muiTheme";
import "./styles.css";

// 页面入口只安装一次公共主题和基础样式；原生状态加载及命令订阅由壳层应用拥有其生命周期。
ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <ThemeProvider theme={hubTheme}>
      <CssBaseline />
      <App />
    </ThemeProvider>
  </React.StrictMode>,
);
