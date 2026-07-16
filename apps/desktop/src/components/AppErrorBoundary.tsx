import React, { type ReactNode } from "react";

function getErrorMessage(error: unknown): string {
  if (error instanceof Error) {
    return error.message;
  }

  if (typeof error === "string") {
    return error;
  }

  try {
    return JSON.stringify(error) ?? "未知错误";
  } catch {
    return "未知错误";
  }
}

interface Props {
  children: ReactNode;
}

interface State {
  error: unknown;
}

export class AppErrorBoundary extends React.Component<Props, State> {
  private hasLoggedError = false;

  constructor(props: Props) {
    super(props);
    this.state = { error: null };
  }

  static getDerivedStateFromError(error: unknown): State {
    return { error };
  }

  componentDidCatch(error: unknown, info: React.ErrorInfo): void {
    if (!this.hasLoggedError) {
      this.hasLoggedError = true;
      console.error("应用渲染失败", error, info.componentStack);
    }
  }

  render(): ReactNode {
    if (this.state.error !== null) {
      return (
        <div className="app-error-boundary" role="alert">
          <h1>应用发生错误</h1>
          <p>{getErrorMessage(this.state.error)}</p>
          <button
            type="button"
            className="reload-button"
            onClick={() => window.location.reload()}
          >
            重新加载应用
          </button>
        </div>
      );
    }

    return this.props.children;
  }
}
