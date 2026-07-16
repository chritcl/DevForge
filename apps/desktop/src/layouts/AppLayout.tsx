import { NavLink, Outlet } from "react-router";

import { useUIStore } from "../stores/ui";

export function AppLayout() {
  const sidebarCollapsed = useUIStore(
    (state) => state.sidebarCollapsed,
  );

  const toggleSidebar = useUIStore(
    (state) => state.toggleSidebar,
  );

  return (
    <div
      className="app-layout"
      data-sidebar-collapsed={sidebarCollapsed}
    >
      <aside
        className="activity-bar"
        aria-label="主导航"
      >
        <button
          type="button"
          className="sidebar-toggle"
          aria-expanded={!sidebarCollapsed}
          aria-label={
            sidebarCollapsed ? "展开主导航" : "收起主导航"
          }
          onClick={toggleSidebar}
        >
          {sidebarCollapsed ? "展开" : "收起"}
        </button>

        <nav className="primary-navigation">
          <NavLink
            to="/"
            end
            aria-label="首页"
          >
            <span className="navigation-label">首页</span>
          </NavLink>

          <NavLink
            to="/settings"
            aria-label="设置"
          >
            <span className="navigation-label">设置</span>
          </NavLink>
        </nav>
      </aside>

      <main className="main-content">
        <Outlet />
      </main>
    </div>
  );
}
