import { createHashRouter } from "react-router";

import App from "./App";
import { NotFoundPage } from "./pages/NotFoundPage";
import { RouteErrorPage } from "./pages/RouteErrorPage";
import { SettingsPage } from "./pages/SettingsPage";
import { WorkspaceListPage } from "./pages/WorkspaceListPage";
import { WorkspacePage } from "./pages/WorkspacePage";

export const router = createHashRouter([
  {
    path: "/",
    Component: App,
    ErrorBoundary: RouteErrorPage,
    children: [
      {
        index: true,
        Component: WorkspaceListPage,
      },
      {
        path: "workspace/:id",
        Component: WorkspacePage,
      },
      {
        path: "settings",
        Component: SettingsPage,
      },
      {
        path: "*",
        Component: NotFoundPage,
      },
    ],
  },
]);
