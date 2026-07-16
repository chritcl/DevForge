import { Link, isRouteErrorResponse, useRouteError } from "react-router";

function getErrorMessage(error: unknown): string {
  if (isRouteErrorResponse(error)) {
    return error.statusText || `${error.status}`;
  }

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

export function RouteErrorPage() {
  const error = useRouteError();

  return (
    <div className="error-page" role="alert">
      <h1>页面发生错误</h1>
      <p>{getErrorMessage(error)}</p>
      <div className="error-actions">
        <Link to="/" className="error-link">
          返回首页
        </Link>
        <button
          type="button"
          className="reload-button"
          onClick={() => window.location.reload()}
        >
          重新加载应用
        </button>
      </div>
    </div>
  );
}
