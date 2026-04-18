import { NavLink } from "react-router-dom";
import {
  LayoutDashboard,
  Globe,
  Code2,
  Hexagon,
  Database,
  Bug,
  FileText,
  Mail,
  Settings,
} from "lucide-react";

const navItems = [
  { to: "/", icon: LayoutDashboard, label: "Dashboard" },
  { to: "/sites", icon: Globe, label: "Sites" },
  { to: "/php", icon: Code2, label: "PHP" },
  { to: "/node", icon: Hexagon, label: "Node.js" },
  { to: "/services", icon: Database, label: "Services" },
  { to: "/dumps", icon: Bug, label: "Dumps" },
  { to: "/logs", icon: FileText, label: "Logs" },
  { to: "/mail", icon: Mail, label: "Mail" },
  { to: "/settings", icon: Settings, label: "Settings" },
];

export function Sidebar() {
  return (
    <aside className="w-60 bg-sidebar-bg text-white flex flex-col h-full shrink-0">
      {/* Logo */}
      <div className="px-6 py-5 border-b border-white/10">
        <h1 className="text-xl font-bold tracking-tight">
          <span className="text-indigo-400">php</span>Herd
        </h1>
        <p className="text-xs text-white/40 mt-0.5">Development Environment</p>
      </div>

      {/* Navigation */}
      <nav className="flex-1 px-3 py-4 space-y-1 overflow-y-auto">
        {navItems.map((item) => (
          <NavLink
            key={item.to}
            to={item.to}
            end={item.to === "/"}
            className={({ isActive }) =>
              `flex items-center gap-3 px-3 py-2.5 rounded-lg text-sm font-medium transition-colors ${
                isActive
                  ? "bg-sidebar-active text-white"
                  : "text-white/60 hover:bg-sidebar-hover hover:text-white/90"
              }`
            }
          >
            <item.icon className="w-5 h-5 shrink-0" />
            {item.label}
          </NavLink>
        ))}
      </nav>

      {/* Footer */}
      <div className="px-6 py-4 border-t border-white/10">
        <div className="flex items-center gap-2">
          <div className="w-2 h-2 rounded-full bg-success" />
          <span className="text-xs text-white/50">All systems running</span>
        </div>
        <p className="text-xs text-white/30 mt-1">v0.1.0</p>
      </div>
    </aside>
  );
}
