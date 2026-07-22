import { useEffect, useState } from "react";
import { getStreak, getDue, localDayString, type ConceptProgress } from "../lib/ipc";
import Button from "../components/ui/Button";
import Tara, { type TaraState } from "../components/Tara";

// Derived from the data already on screen, not a tracked "freeze protection"
// signal from the backend: an active streak with something due today reads
// as "attentive" (there's something to do to keep it going); otherwise she's
// "content" — including a broken/fresh-start streak, since re-entry is
// neutral, never a sad state (see the mascot spec's anti-guilt-pattern rule).
function homeIdleState(streak: number, dueCount: number): TaraState {
  if (streak > 0 && dueCount > 0) return "attentive";
  return "content";
}

export default function Home({
  t,
  onGoToSubjects,
}: {
  t: (k: string) => string;
  onGoToSubjects: () => void;
}) {
  const [streak, setStreak] = useState(0);
  const [due, setDue] = useState<ConceptProgress[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    setLoading(true);
    Promise.all([
      getStreak(localDayString()).catch(() => 0),
      getDue(Math.floor(Date.now() / 1000)).catch(() => []),
    ]).then(([s, d]) => {
      setStreak(s);
      setDue(d);
      setLoading(false);
    });
  }, []);

  return (
    <div className="max-w-lg mx-auto">
      <div className="flex items-center gap-2 mb-1">
        <Tara state={homeIdleState(streak, due.length)} size={40} />
        <span className="font-display text-3xl text-celebration">
          🔥 {streak}
        </span>
      </div>
      <p className="text-ink-muted text-sm mb-6">{t("streakDays")}</p>

      <div className="border border-border rounded-xl p-5">
        <h2 className="font-display text-lg mb-2">{t("todaysMission")}</h2>
        {loading ? (
          <p className="text-ink-muted text-sm">{t("loading")}</p>
        ) : due.length > 0 ? (
          <p className="text-sm mb-4">
            {due.length} {t("dueCount")}
          </p>
        ) : (
          <p className="text-sm mb-4">{t("allCaughtUp")}</p>
        )}
        <Button onClick={onGoToSubjects}>{t("goToSubjects")}</Button>
      </div>
    </div>
  );
}
