import Tooltip from "./ui/Tooltip";

export type Tab = "home" | "subjects" | "tutor" | "progress" | "settings";

const ICON: Record<Tab, string> = {
  home: "⌂",
  subjects: "▤",
  tutor: "💬",
  progress: "★",
  settings: "⚙",
};

export default function NavRail({
  tab,
  onTabChange,
  t,
}: {
  tab: Tab;
  onTabChange: (t: Tab) => void;
  t: (k: string) => string;
}) {
  const items: Tab[] = ["home", "subjects", "tutor", "progress", "settings"];

  return (
    <nav className="flex flex-col items-center gap-2 w-16 py-4 bg-surface border-r border-border">
      {items.map((item) => (
        <Tooltip key={item} label={t(item)}>
          <button
            aria-label={t(item)}
            aria-current={tab === item ? "page" : undefined}
            onClick={() => onTabChange(item)}
            className={`w-11 h-11 rounded-lg flex items-center justify-center text-lg transition-colors ${
              tab === item
                ? "bg-primary text-white"
                : "text-ink-muted hover:bg-border"
            }`}
          >
            {ICON[item]}
          </button>
        </Tooltip>
      ))}
    </nav>
  );
}
