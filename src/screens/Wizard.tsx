import { useEffect, useState } from "react";
import {
  completeWizard,
  listSubjects,
  listChapters,
  startQuiz,
  submitAnswer,
  finishQuiz,
  localDayString,
  type Settings,
  type Question,
  type AnswerReview,
} from "../lib/ipc";
import { useT, type Lang } from "../lib/i18n";
import Button from "../components/ui/Button";
import Tara from "../components/Tara";

const GRADES = [6, 7, 8, 9, 10, 11, 12];
// Only grade with populated content in MVP-0; independent of whatever grade
// is chosen on the Grade screen (see design spec section 2, honesty note).
const WARMUP_GRADE = 8;

type Step = "welcome" | "grade" | "warmup" | "celebrate";

function WarmupQuestion({
  t,
  onAnswered,
}: {
  t: (k: string) => string;
  onAnswered: () => void;
}) {
  const [sessionId, setSessionId] = useState<number | null>(null);
  const [question, setQuestion] = useState<Question | null>(null);
  const [chosen, setChosen] = useState<number | null>(null);
  const [review, setReview] = useState<AnswerReview | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    (async () => {
      try {
        const subjects = await listSubjects(WARMUP_GRADE);
        const subject = subjects[0];
        const chapters = subject ? await listChapters(subject.id) : [];
        const chapter = chapters[0];
        if (!chapter) throw new Error("no warmup chapter available");
        const [sid, questions] = await startQuiz(chapter.id);
        if (cancelled) return;
        setSessionId(sid);
        setQuestion(questions[0] ?? null);
      } catch (e) {
        if (!cancelled) setError(String(e));
      } finally {
        if (!cancelled) setLoading(false);
      }
    })();
    return () => {
      cancelled = true;
    };
  }, []);

  async function choose(optionId: number) {
    if (review || sessionId == null || !question) return;
    setChosen(optionId);
    try {
      const r = await submitAnswer(sessionId, question.id, optionId, localDayString());
      setReview(r);
      // Closes the single-question session; its QuizResult isn't shown here
      // (see design spec section 4 — a 1-question score screen would be a
      // strange first impression).
      await finishQuiz(sessionId);
    } catch (e) {
      setError(String(e));
    }
  }

  if (loading) {
    return (
      <div className="min-h-full flex flex-col items-center justify-center gap-4 px-6">
        <Tara state="welcoming" size={56} />
        <p className="text-ink-muted text-sm">{t("loading")}</p>
      </div>
    );
  }

  if (error || !question) {
    return (
      <div className="min-h-full flex flex-col items-center justify-center gap-4 px-6 text-center">
        <p className="text-red-600 text-sm">{error ?? t("errorGeneric")}</p>
        <Button onClick={onAnswered}>{t("continue")}</Button>
      </div>
    );
  }

  return (
    <div className="min-h-full flex flex-col items-center justify-center gap-5 px-6 text-center">
      <Tara state="welcoming" size={56} />
      <h3 className="font-display text-lg max-w-sm">{question.prompt}</h3>
      <div className="flex flex-col gap-2 w-full max-w-sm">
        {question.options.map((o) => {
          let cls =
            "text-left px-3 py-2.5 rounded-lg border border-border bg-surface";
          if (review) {
            if (o.id === review.correctOptionId)
              cls = "text-left px-3 py-2.5 rounded-lg border border-green-600 bg-green-50";
            else if (o.id === chosen)
              cls = "text-left px-3 py-2.5 rounded-lg border border-red-600 bg-red-50";
          }
          return (
            <button
              key={o.id}
              className={cls}
              disabled={!!review}
              onClick={() => choose(o.id)}
            >
              {o.body}
            </button>
          );
        })}
      </div>
      {review && (
        <div
          className={
            review.correct
              ? "px-3 py-2.5 rounded-lg bg-green-50 text-green-800 text-sm max-w-sm"
              : "px-3 py-2.5 rounded-lg bg-red-50 text-red-800 text-sm max-w-sm"
          }
          role="status"
          aria-live="polite"
        >
          {review.correct ? review.explanation : `${review.explanation} ${review.trap}`}
        </div>
      )}
      {review && <Button onClick={onAnswered}>{t("continue")}</Button>}
    </div>
  );
}

export default function Wizard({ onDone }: { onDone: (s: Settings) => void }) {
  const [step, setStep] = useState<Step>("welcome");
  const [language, setLanguage] = useState<Lang>("en");
  const [grade, setGrade] = useState(8);
  // Backend truth (onboarded=true) lands via completeWizard at the end of
  // the Grade screen, but the parent isn't told (onDone) until the
  // celebration screen finishes — see design spec section 5.
  const [settingsResult, setSettingsResult] = useState<Settings | null>(null);
  const [error, setError] = useState<string | null>(null);
  const t = useT(language);

  function chooseLanguage(l: Lang) {
    setLanguage(l);
    setStep("grade");
  }

  async function confirmGrade() {
    setError(null);
    try {
      const s = await completeWizard(language, grade, "CBSE");
      setSettingsResult(s);
      setStep("warmup");
    } catch (e) {
      setError(String(e));
    }
  }

  // No t() here: no language has been chosen yet on this screen, so there is
  // nothing to look up (see design spec section 7) — this is intentional,
  // not a missed translation.
  if (step === "welcome") {
    return (
      <div className="min-h-full flex flex-col items-center justify-center gap-6 px-6 text-center">
        <Tara state="welcoming" size={112} />
        <h1 className="font-display text-2xl">Sarvo Shala</h1>
        <div className="flex gap-3">
          <Button onClick={() => chooseLanguage("en")}>English</Button>
          <Button onClick={() => chooseLanguage("hi")}>हिन्दी</Button>
        </div>
      </div>
    );
  }

  if (step === "grade") {
    return (
      <div className="min-h-full flex flex-col items-center justify-center gap-6 px-6">
        <Tara state="welcoming" size={48} />
        <h2 className="font-display text-lg">{t("chooseGrade")}</h2>
        <div className="flex flex-wrap gap-2 justify-center max-w-sm">
          {GRADES.map((g) => (
            <button
              key={g}
              onClick={() => setGrade(g)}
              className={`w-11 h-11 rounded-full border text-sm ${
                g === grade
                  ? "bg-primary text-white border-primary"
                  : "bg-surface text-ink border-border"
              }`}
            >
              {g}
            </button>
          ))}
        </div>
        {error && <p className="text-red-600 text-sm">{error}</p>}
        <Button onClick={confirmGrade}>{t("continue")}</Button>
      </div>
    );
  }

  if (step === "warmup") {
    return <WarmupQuestion t={t} onAnswered={() => setStep("celebrate")} />;
  }

  return (
    <div className="min-h-full flex flex-col items-center justify-center gap-6 px-6 text-center">
      <Tara state="celebrating" size={96} />
      <h2 className="font-display text-2xl text-celebration">
        {t("day1StreakTitle")}
      </h2>
      <Button onClick={() => settingsResult && onDone(settingsResult)}>
        {t("continue")}
      </Button>
    </div>
  );
}
