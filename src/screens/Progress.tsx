import { useEffect, useState } from "react";
import {
  getProgress,
  getDue,
  getStreak,
  getGamification,
  type ConceptProgress,
  type GamificationSummary,
} from "../lib/ipc";
import type { Chapter } from "../lib/ipc";
import MasteryGrid from "../components/MasteryGrid";
import { badgeLabel } from "../lib/gamification";

function todayString(): string {
  const d = new Date();
  const y = d.getFullYear();
  const m = String(d.getMonth() + 1).padStart(2, "0");
  const day = String(d.getDate()).padStart(2, "0");
  return `${y}-${m}-${day}`;
}

export default function Progress({
  chapter,
  t,
}: {
  chapter: Chapter | null;
  t: (k: string) => string;
}) {
  const [progress, setProgress] = useState<ConceptProgress[]>([]);
  const [due, setDue] = useState<ConceptProgress[]>([]);
  const [streak, setStreak] = useState(0);
  const [gamification, setGamification] = useState<GamificationSummary | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    setLoading(true);
    let pending = 4;
    const done = () => {
      pending -= 1;
      if (pending === 0) setLoading(false);
    };
    getProgress(chapter ? chapter.id : null)
      .then(setProgress)
      .catch(() => setProgress([]))
      .finally(done);
    getDue(Math.floor(Date.now() / 1000))
      .then(setDue)
      .catch(() => setDue([]))
      .finally(done);
    getStreak(todayString())
      .then(setStreak)
      .catch(() => setStreak(0))
      .finally(done);
    getGamification()
      .then(setGamification)
      .catch(() => setGamification(null))
      .finally(done);
  }, [chapter]);

  return (
    <div className="max-w-2xl mx-auto">
      <div className="flex items-center gap-4 mb-4 flex-wrap">
        <div className="font-display text-2xl font-semibold">
          🔥 {streak}
        </div>
        {gamification && (
          <>
            <div className="font-display text-lg text-ink">
              {t("level")} {gamification.level}
            </div>
            <div className="text-sm text-ink-muted">
              {gamification.xp} XP — {gamification.xpIntoLevel}/{gamification.xpForNextLevel} {t("xpToNextLevel")}
            </div>
            {gamification.badges.length > 0 && (
              <div className="flex gap-1 flex-wrap">
                {gamification.badges.map((b) => (
                  <span
                    key={b}
                    className="text-xs bg-primary/10 text-primary px-2 py-0.5 rounded-full font-medium"
                  >
                    {badgeLabel(b, t)}
                  </span>
                ))}
              </div>
            )}
          </>
        )}
      </div>

      <h3 className="font-display text-lg mb-2">{t("mastery")}</h3>
      {progress.length > 0 ? (
        <MasteryGrid concepts={progress} t={t} />
      ) : (
        !loading && <p className="text-ink-muted text-sm">{t("noDue")}</p>
      )}

      <h3 className="font-display text-lg mb-2">{t("dueForReview")}</h3>
      {loading ? (
        <p className="text-ink-muted">{t("loading")}</p>
      ) : due.length === 0 ? (
        <p className="text-ink-muted">{t("noDue")}</p>
      ) : (
        <ul className="pl-4">
          {due.map((c) => (
            <li key={c.conceptId} className="text-ink">
              {c.name}
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
