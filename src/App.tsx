import { useEffect, useState } from "react";
import { getSettings, updateLanguage, type Settings, type Chapter } from "./lib/ipc";
import { useT, type Lang } from "./lib/i18n";
import AppShell from "./components/AppShell";
import type { Tab } from "./components/NavRail";
import Wizard from "./screens/Wizard";
import Home from "./screens/Home";
import Browse from "./screens/Browse";
import Quiz from "./screens/Quiz";
import Progress from "./screens/Progress";
import Tutor from "./screens/Tutor";
import SettingsScreen from "./screens/Settings";

export default function App() {
  const [settings, setSettings] = useState<Settings | null>(null);
  const [tab, setTab] = useState<Tab>("home");
  const [selectedChapter, setSelectedChapter] = useState<Chapter | null>(null);
  const [quizChapter, setQuizChapter] = useState<Chapter | null>(null);
  const t = useT((settings?.language as Lang) ?? "en");

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

  const openTutor = (c: Chapter) => {
    setSelectedChapter(c);
    setTab("tutor");
  };
  const openQuiz = (c: Chapter) => {
    setQuizChapter(c);
  };

  async function changeLanguage(l: Lang) {
    setSettings((s) => (s ? { ...s, language: l } : s));
    try {
      const updated = await updateLanguage(l);
      setSettings(updated);
    } catch {
      // backend command lands in a follow-on plan; local UI state already updated
    }
  }

  return (
    <div className="h-full">
      <AppShell tab={tab} onTabChange={setTab} t={t}>
        {tab === "home" && (
          <Home t={t} onGoToSubjects={() => setTab("subjects")} />
        )}
        {tab === "subjects" && (
          <Browse
            grade={settings.grade}
            t={t}
            language={settings.language}
            onAskTutor={openTutor}
            onStartQuiz={openQuiz}
          />
        )}
        {tab === "tutor" &&
          (selectedChapter ? (
            <Tutor chapter={selectedChapter} t={t} />
          ) : (
            <p>{t("subjects")}</p>
          ))}
        {tab === "progress" && <Progress chapter={selectedChapter} t={t} />}
        {tab === "settings" && (
          <SettingsScreen
            language={settings.language as Lang}
            onLanguageChange={changeLanguage}
            t={t}
          />
        )}
      </AppShell>
      {quizChapter && (
        <div className="fixed inset-0 bg-ink/40 flex items-center justify-center">
          <div className="bg-surface rounded-xl p-6 max-w-lg w-full">
            <Quiz chapter={quizChapter} t={t} />
            <button
              className="mt-4 text-sm text-ink-muted underline"
              onClick={() => setQuizChapter(null)}
            >
              {t("finish")}
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
