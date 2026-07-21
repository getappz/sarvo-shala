import { useState } from "react";
import {
  startQuiz,
  submitAnswer,
  finishQuiz,
  localDayString,
  type Question,
  type AnswerReview,
  type QuizResult,
  type Chapter,
} from "../lib/ipc";
import Button from "../components/ui/Button";

type Phase = "intro" | "question" | "done";

interface Feedback {
  review: AnswerReview;
  chosen: number;
}

export default function Quiz({
  chapter,
  t,
}: {
  chapter: Chapter;
  t: (k: string) => string;
}) {
  const [phase, setPhase] = useState<Phase>("intro");
  const [questions, setQuestions] = useState<Question[]>([]);
  const [sessionId, setSessionId] = useState(0);
  const [index, setIndex] = useState(0);
  const [chosen, setChosen] = useState<number | null>(null);
  const [feedback, setFeedback] = useState<Feedback | null>(null);
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [result, setResult] = useState<QuizResult | null>(null);

  async function begin() {
    setError(null);
    try {
      const [sid, qs] = await startQuiz(chapter.id);
      setSessionId(sid);
      setQuestions(qs);
      setIndex(0);
      setChosen(null);
      setFeedback(null);
      setPhase("question");
    } catch {
      setError(t("errorGeneric"));
    }
  }

  async function choose(optId: number) {
    if (feedback || submitting) return;
    setError(null);
    setSubmitting(true);
    try {
      const review = await submitAnswer(
        sessionId,
        questions[index].id,
        optId,
        localDayString()
      );
      setChosen(optId);
      setFeedback({ review, chosen: optId });
    } catch {
      setError(t("errorGeneric"));
    } finally {
      setSubmitting(false);
    }
  }

  function next() {
    if (index + 1 < questions.length) {
      setIndex(index + 1);
      setChosen(null);
      setFeedback(null);
    } else {
      void finish();
    }
  }

  async function finish() {
    setError(null);
    try {
      const r = await finishQuiz(sessionId);
      setResult(r);
      setPhase("done");
    } catch {
      setError(t("errorGeneric"));
    }
  }

  if (phase === "intro") {
    return (
      <div className="max-w-xl mx-auto">
        <h3 className="font-display text-lg mb-3">{chapter.title}</h3>
        <Button onClick={begin}>{t("startQuiz")}</Button>
        {error && <p className="error mt-2">{error}</p>}
      </div>
    );
  }

  if (phase === "done" && result) {
    return (
      <div className="max-w-xl mx-auto">
        <h3 className="font-display text-lg mb-2">
          {t("score")}: {Math.round(result.score)}%
        </h3>
        <p className="text-sm text-ink-muted mb-3">
          {result.correct} / {result.total}
        </p>
        <ul className="list-none p-0 flex flex-col gap-1.5 mb-4">
          {result.perQuestion.map((r) => (
            <li
              key={r.questionId}
              className={r.correct ? "text-green-800" : "text-red-800"}
            >
              {r.correct ? "✓" : "✗"} {r.explanation}
            </li>
          ))}
        </ul>
        <Button onClick={() => setPhase("intro")}>{t("finish")}</Button>
      </div>
    );
  }

  const q = questions[index];
  return (
    <div className="max-w-xl mx-auto">
      <div className="text-xs text-ink-muted mb-2">
        {index + 1} / {questions.length}
      </div>
      <h3 className="font-display text-lg mb-3">{q.prompt}</h3>
      <div className="flex flex-col gap-2 my-3">
        {q.options.map((o) => {
          let cls =
            "text-left px-3 py-2.5 rounded-lg border border-border bg-surface";
          if (feedback) {
            if (o.id === feedback.review.correctOptionId)
              cls = "text-left px-3 py-2.5 rounded-lg border border-green-600 bg-green-50";
            else if (o.id === chosen)
              cls = "text-left px-3 py-2.5 rounded-lg border border-red-600 bg-red-50";
          }
          return (
            <button
              key={o.id}
              className={cls}
              disabled={!!feedback || submitting}
              onClick={() => choose(o.id)}
            >
              {o.body}
            </button>
          );
        })}
      </div>
      {feedback && (
        <div
          className={
            feedback.review.correct
              ? "px-3 py-2.5 rounded-lg mb-3 bg-green-50 text-green-800"
              : "px-3 py-2.5 rounded-lg mb-3 bg-red-50 text-red-800"
          }
          role="status"
          aria-live="polite"
        >
          {feedback.review.correct
            ? feedback.review.explanation
            : `${feedback.review.explanation} ${feedback.review.trap}`}
        </div>
      )}
      {error && <p className="error mb-3">{error}</p>}
      {feedback && (
        <Button onClick={next}>
          {index + 1 < questions.length ? t("next") : t("finish")}
        </Button>
      )}
    </div>
  );
}
