import type { DbStatus } from "../bindings";

interface HealthStatusProps {
  dbStatus: DbStatus;
}

function assertNever(value: never): never {
  throw new Error(`未知数据库状态：${JSON.stringify(value)}`);
}

function getStatusLabel(dbStatus: DbStatus): string {
  switch (dbStatus.type) {
    case "NotInitialized":
      return "未初始化";
    case "Ready":
      return `就绪（migration v${dbStatus.migration_version}）`;
    case "Error":
      return `错误：${dbStatus.message}`;
    default:
      return assertNever(dbStatus);
  }
}

export function HealthStatus({ dbStatus }: HealthStatusProps) {
  return (
    <span role="status" data-status={dbStatus.type}>
      {getStatusLabel(dbStatus)}
    </span>
  );
}
