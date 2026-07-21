import { useEffect, useState } from "react";
import {
  listSubjects,
  listChapters,
  listConcepts,
  type Subject,
  type Chapter,
  type Concept,
} from "../lib/ipc";
import Button from "../components/ui/Button";

export default function Browse({
  grade,
  t,
  language,
  onAskTutor,
  onStartQuiz,
}: {
  grade: number;
  t: (k: string) => string;
  language: string;
  onAskTutor: (c: Chapter) => void;
  onStartQuiz: (c: Chapter) => void;
}) {
  const [subjects, setSubjects] = useState<Subject[]>([]);
  const [subjectId, setSubjectId] = useState<number | null>(null);
  const [chapters, setChapters] = useState<Chapter[]>([]);
  const [chapterId, setChapterId] = useState<number | null>(null);
  const [concepts, setConcepts] = useState<Concept[]>([]);

  useEffect(() => {
    listSubjects(grade)
      .then((subs) => {
        setSubjects(subs);
        if (subs.length > 0) setSubjectId(subs[0].id);
      })
      .catch(() => setSubjects([]));
  }, [grade]);

  useEffect(() => {
    if (subjectId == null) return;
    listChapters(subjectId)
      .then((chs) => {
        setChapters(chs);
        setChapterId(chs.length > 0 ? chs[0].id : null);
      })
      .catch(() => setChapters([]));
  }, [subjectId]);

  useEffect(() => {
    if (chapterId == null) return;
    listConcepts(chapterId)
      .then(setConcepts)
      .catch(() => setConcepts([]));
  }, [chapterId]);

  const activeChapter = chapters.find((c) => c.id === chapterId) ?? null;

  return (
    <div className="max-w-4xl mx-auto">
      <div className="flex gap-2 mb-4">
        {subjects.map((s) => (
          <button
            key={s.id}
            onClick={() => setSubjectId(s.id)}
            className={`px-4 py-1.5 rounded-full border text-sm transition-colors ${
              s.id === subjectId
                ? "bg-primary text-white border-primary"
                : "bg-surface text-ink border-border"
            }`}
          >
            {language === "hi" ? s.nameHi || s.name : s.name}
          </button>
        ))}
      </div>

      <div className="grid grid-cols-[280px_1fr] gap-4">
        <ul className="flex flex-col gap-2 list-none m-0 p-0">
          {chapters.map((c) => (
            <li key={c.id}>
              <button
                onClick={() => setChapterId(c.id)}
                className={`w-full text-left flex flex-col gap-1 px-3 py-2.5 rounded-lg border ${
                  c.id === chapterId
                    ? "border-primary bg-primary/5"
                    : "border-border bg-surface"
                }`}
              >
                <strong className="text-ink">
                  {language === "hi" ? c.titleHi || c.title : c.title}
                </strong>
                <span className="text-xs text-ink-muted">{c.summary}</span>
              </button>
            </li>
          ))}
        </ul>

        {activeChapter && (
          <div>
            <h3 className="font-display text-lg mb-3">
              {language === "hi"
                ? activeChapter.titleHi || activeChapter.title
                : activeChapter.title}
            </h3>
            <div className="grid grid-cols-[repeat(auto-fill,minmax(200px,1fr))] gap-3">
              {concepts.map((cn) => (
                <div
                  key={cn.id}
                  className="border border-border rounded-lg p-3 bg-surface"
                >
                  <div className="font-semibold mb-1.5">{cn.name}</div>
                  {cn.keyFormula && (
                    <span className="inline-block bg-border/60 px-2 py-0.5 rounded font-mono text-xs">
                      {cn.keyFormula}
                    </span>
                  )}
                  <div className="my-1.5 text-primary text-xs">
                    {"●".repeat(cn.examWeight)}
                    <span className="text-ink-muted"> {t("weight")}</span>
                  </div>
                  {cn.commonError && (
                    <div className="text-xs text-amber-800 bg-amber-50 px-1.5 py-1 rounded">
                      ⚠ {cn.commonError}
                    </div>
                  )}
                </div>
              ))}
            </div>
            <div className="flex gap-2.5 mt-3.5">
              <Button onClick={() => onAskTutor(activeChapter)}>{t("tutor")}</Button>
              <Button onClick={() => onStartQuiz(activeChapter)}>{t("startQuiz")}</Button>
            </div>
          </div>
        )}
      </div>

      <footer className="mt-4 text-xs text-ink-muted text-center">
        {t("sampleContentNotice")}
      </footer>
    </div>
  );
}
