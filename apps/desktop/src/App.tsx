import { useThemeSync } from "./hooks/useThemeSync";
import { AppLayout } from "./layouts/AppLayout";

export default function App() {
  useThemeSync();

  return <AppLayout />;
}
