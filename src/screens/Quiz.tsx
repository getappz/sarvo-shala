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
      <div className="quiz">
        <h3>{chapter.title}</h3>
        <button className="primary" onClick={begin}>
          {t("startQuiz")}
        </button>
        {error && <p className="error">{error}</p>}
      </div>
    );
  }

  if (phase === "done" && result) {
    return (
      <div className="quiz">
        <h3>{t("score")}: {Math.round(result.score)}%</h3>
        <p>
          {result.correct} / {result.total}
        </p>
        <ul className="review-list">
          {result.perQuestion.map((r) => (
            <li key={r.questionId} className={r.correct ? "ok" : "bad"}>
              {r.correct ? "✓" : "✗"} {r.explanation}
            </li>
          ))}
        </ul>
        <button className="primary" onClick={() => setPhase("intro")}>
          {t("finish")}
        </button>
      </div>
    );
  }

  const q = questions[index];
  return (
    <div className="quiz">
      <div className="quiz-progress">
        {index + 1} / {questions.length}
      </div>
      <h3>{q.prompt}</h3>
      <div className="options">
        {q.options.map((o) => {
          let cls = "option";
          if (feedback) {
            if (o.id === feedback.review.correctOptionId) cls += " correct";
            else if (o.id === chosen) cls += " wrong";
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
          className={feedback.review.correct ? "explain ok" : "explain bad"}
          role="status"
          aria-live="polite"
        >
          {feedback.review.correct
            ? feedback.review.explanation
            : `${feedback.review.explanation} ${feedback.review.trap}`}
        </div>
      )}
      {error && <p className="error">{error}</p>}
      {feedback && (
        <button className="primary" onClick={next}>
          {index + 1 < questions.length ? t("next") : t("finish")}
        </button>
      )}
    </div>
  );
}
