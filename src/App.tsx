import { useEffect, useState } from "react";
import { getSettings, type Settings, type Chapter } from "./lib/ipc";
import { useT } from "./lib/i18n";
import Wizard from "./screens/Wizard";
import Browse from "./screens/Browse";
import Quiz from "./screens/Quiz";
import Progress from "./screens/Progress";

type Tab = "browse" | "tutor" | "quiz" | "progress";

export default function App() {
  const [settings, setSettings] = useState<Settings | null>(null);
  const [tab, setTab] = useState<Tab>("browse");
  const [selectedChapter, setSelectedChapter] = useState<Chapter | null>(null);
  const t = useT((settings?.language as "en" | "hi") ?? "en");

  useEffect(() => {
    getSettings()
      .then(setSettings)
      .catch(() =>
        setSettings({ language: "en", grade: 8, board: "CBSE", onboarded: false })
      );
  }, []);

  if (!settings) return <p className="center">{t("loading")}</p>;

  if (!settings.onboarded) {
    return <Wizard onDone={setSettings} />;
  }

  const tabs: Tab[] = ["browse", "tutor", "quiz", "progress"];

  const openTutor = (c: Chapter) => {
    setSelectedChapter(c);
    setTab("tutor");
  };
  const openQuiz = (c: Chapter) => {
    setSelectedChapter(c);
    setTab("quiz");
  };

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
        {tab === "browse" && (
          <Browse
            grade={settings.grade}
            t={t}
            onAskTutor={openTutor}
            onStartQuiz={openQuiz}
          />
        )}
        {tab === "quiz" &&
          (selectedChapter ? (
            <Quiz chapter={selectedChapter} t={t} />
          ) : (
            <p>{t("browse")}</p>
          ))}

        {tab === "progress" && <Progress chapter={selectedChapter} t={t} />}

        {tab === "tutor" && (
          <p>
            {selectedChapter
              ? `${t(tab)} — ${selectedChapter.title} (${t("loading")})`
              : t("browse")}
          </p>
        )}
      </main>
    </div>
  );
}
