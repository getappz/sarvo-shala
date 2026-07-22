import type { ConceptProgress } from "../lib/ipc";
import { masteryLevel, type MasteryLevel } from "../lib/mastery";
import "../styles/mastery-grid.css";

export default function MasteryGrid({
  concepts,
  t,
}: {
  concepts: ConceptProgress[];
  t: (k: string) => string;
}) {
  return (
    <div className="flex flex-col gap-1.5 mb-6" role="grid" aria-label={t("mastery")}>
      {concepts.map((c) => {
        const ml = masteryLevel(c.attempts, c.pKnown);
        return (
          <div
            key={c.conceptId}
            className="grid grid-cols-[minmax(100px,160px)_minmax(0,1fr)_44px] items-center gap-2"
            role="row"
          >
            <span className="text-sm text-ink">{c.name}</span>
            <MasteryCell level={ml} t={t} />
            <span className="text-xs text-ink-muted text-right">
              {Math.round(c.pKnown * 100)}%
            </span>
          </div>
        );
      })}
    </div>
  );
}

export function MasteryCell({
  level,
  t,
}: {
  level: MasteryLevel;
  t: (k: string) => string;
}) {
  const keyMap: Record<MasteryLevel, string> = {
    not_started: "masteryNotStarted",
    learning: "masteryLearning",
    practiced: "masteryPracticed",
    mastered: "masteryMastered",
  };
  const label = t(keyMap[level]);
  const colorVar = masteryColor(level);
  return (
    <div
      role="img"
      aria-label={label}
      className={`mastery-cell mastery-cell--${level}`}
      style={{ backgroundColor: `var(${colorVar})` }}
    />
  );
}

export function masteryColor(level: MasteryLevel): string {
  const map: Record<MasteryLevel, string> = {
    not_started: "--color-mastery-1",
    learning: "--color-mastery-2",
    practiced: "--color-mastery-3",
    mastered: "--color-mastery-4",
  };
  return map[level];
}
