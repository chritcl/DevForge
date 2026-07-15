/**
 * 应用级 Query Key 工厂
 *
 * 所有 AppInfo 查询必须通过此工厂创建，
 * 不得在组件或 Hook 中散落手写 Query Key。
 */
export const appKeys = {
  all: ["app"] as const,
  info: () => [...appKeys.all, "info"] as const,
};
