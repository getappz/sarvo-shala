import { useEffect, useState } from "react";
import { getSettings, type Settings } from "./lib/ipc";
import { useT } from "./lib/i18n";
import Wizard from "./screens/Wizard";

type Tab = "browse" | "tutor" | "quiz" | "progress";

export default function App() {
  const [settings, setSettings] = useState<Settings | null>(null);
  const [tab, setTab] = useState<Tab>("browse");
  const t = useT((settings?.language as "en" | "hi") ?? "en");

  useEffect(() => {
    getSettings()
      .then(setSettings)
      .catch(() => setSettings({ language: "en", grade: 8, board: "CBSE", onboarded: false }));
  }, []);

  if (!settings) return <p className="center">{t("loading")}</p>;

  if (!settings.onboarded) {
    return <Wizard onDone={setSettings} />;
  }

  const tabs: Tab[] = ["browse", "tutor", "quiz", "progress"];

  return (
    <div className="app">
      <header className="tabs">
        {tabs.map((tb) => (
          <button
            key={tb}
            className={tb === tab ? "tab active" : "tab"}
            onClick={() => setTab(tb)}
          >
            {t(tb)}
          </button>
        ))}
      </header>
      <main className="content">
        <p>
          {t(tab)} {t("loading")}
        </p>
      </main>
    </div>
  );
}
