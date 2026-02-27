import React, { Component, ReactNode } from "react";
import ReactDOM from "react-dom/client";
import App from "./App";

// Initialize i18n
import "./i18n";


// Initialize model store (loads models and sets up event listeners)
import { useModelStore } from "./stores/modelStore";
useModelStore.getState().initialize();

class ErrorBoundary extends Component<{ children: ReactNode }, { hasError: boolean, error: any, info: any }> {
  constructor(props: { children: ReactNode }) {
    super(props);
    this.state = { hasError: false, error: null, info: null };
  }

  componentDidCatch(error: any, info: any) {
    this.setState({ hasError: true, error, info });
  }

  render() {
    if (this.state.hasError) {
      return (
        <div style={{ padding: 20, color: '#b91c1c', fontFamily: 'monospace', overflow: 'auto', background: '#fee2e2', height: '100vh', width: '100vw' }}>
          <h2>🚨 React App Crashed</h2>
          <p style={{ fontWeight: 'bold' }}>{this.state.error?.toString()}</p>
          <pre style={{ fontSize: '11px', whiteSpace: 'pre-wrap', marginTop: 10 }}>{this.state.info?.componentStack}</pre>
        </div>
      );
    }
    return this.props.children;
  }
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <ErrorBoundary>
      <App />
    </ErrorBoundary>
  </React.StrictMode>,
);
