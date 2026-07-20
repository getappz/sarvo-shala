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
    <div className="progress">
      <div className="streak">
        🔥 {streak}
      </div>

      <h3>{t("mastery")}</h3>
      <div className="mastery-list">
        {progress.map((c) => (
          <div key={c.conceptId} className="mastery-row">
            <span className="mastery-name">{c.name}</span>
            <div className="bar">
              <div
                className="bar-fill"
                style={{ width: `${Math.round(c.pKnown * 100)}%` }}
              />
            </div>
            <span className="mastery-pct">{Math.round(c.pKnown * 100)}%</span>
          </div>
        ))}
      </div>

      <h3>{t("dueForReview")}</h3>
      {loading ? (
        <p className="muted">{t("loading")}</p>
      ) : due.length === 0 ? (
        <p className="muted">{t("noDue")}</p>
      ) : (
        <ul className="due-list">
          {due.map((c) => (
            <li key={c.conceptId}>{c.name}</li>
          ))}
        </ul>
      )}
    </div>
  );
}
