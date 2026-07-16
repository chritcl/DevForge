import { useAppInfo } from "../hooks/useAppInfo";
import { HealthStatus } from "../components/HealthStatus";

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

export function HomePage() {
  const appInfoQuery = useAppInfo();

  if (appInfoQuery.isPending) {
    return <div className="page-loading">加载中...</div>;
  }

  if (appInfoQuery.isError) {
    return (
      <div className="page-error" role="alert">
        <p>加载应用信息失败：{getErrorMessage(appInfoQuery.error)}</p>
        <button
          type="button"
          className="retry-button"
          disabled={appInfoQuery.isFetching}
          onClick={() => void appInfoQuery.refetch()}
        >
          {appInfoQuery.isFetching ? "正在重试..." : "重试"}
        </button>
      </div>
    );
  }

  const data = appInfoQuery.data;

  return (
    <div className="home-page">
      <h1>DevForge</h1>
      <p className="home-subtitle">开发者知识库与 AI 工作台</p>
      <dl className="info-list">
        <dt>版本</dt>
        <dd>{data.version}</dd>
        <dt>数据目录</dt>
        <dd className="data-dir">{data.data_dir}</dd>
        <dt>数据库状态</dt>
        <dd>
          <HealthStatus dbStatus={data.db_status} />
        </dd>
      </dl>
    </div>
  );
}
