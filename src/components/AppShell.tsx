import type { ReactNode } from "react";
import Titlebar from "./Titlebar";
import NavRail, { type Tab } from "./NavRail";

export default function AppShell({
  children,
  tab,
  onTabChange,
  t,
}: {
  children: ReactNode;
  tab: Tab;
  onTabChange: (t: Tab) => void;
  t: (k: string) => string;
}) {
  return (
    <div className="flex flex-col h-full">
      <Titlebar />
      <div className="flex flex-1 min-h-0">
        <NavRail tab={tab} onTabChange={onTabChange} t={t} />
        <main className="flex-1 overflow-auto p-6 bg-surface text-ink font-sans">
          {children}
        </main>
      </div>
    </div>
  );
}
