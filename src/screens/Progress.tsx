import { useEffect, useState } from "react";
import {
  getProgress,
  getDue,
  getStreak,
  type ConceptProgress,
} from "../lib/ipc";
import type { Chapter } from "../lib/ipc";

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
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    setLoading(true);
    let pending = 3;
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
  }, [chapter]);

  return (
    <div className="max-w-2xl mx-auto">
      <div className="font-display text-2xl font-semibold mb-4">🔥 {streak}</div>

      <h3 className="font-display text-lg mb-2">{t("mastery")}</h3>
      <div className="flex flex-col gap-1.5 mb-6">
        {progress.map((c) => (
          <div
            key={c.conceptId}
            className="grid grid-cols-[minmax(100px,160px)_minmax(0,1fr)_44px] items-center gap-2"
          >
            <span className="text-sm text-ink">{c.name}</span>
            <div className="bg-border rounded-md h-2.5 overflow-hidden">
              <div
                className="bg-primary h-full"
                style={{ width: `${Math.round(c.pKnown * 100)}%` }}
              />
            </div>
            <span className="text-xs text-ink-muted text-right">
              {Math.round(c.pKnown * 100)}%
            </span>
          </div>
        ))}
      </div>

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
