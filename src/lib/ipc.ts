import { invoke } from "@tauri-apps/api/core";

export interface Settings {
  language: string;
  grade: number;
  board: string;
  onboarded: boolean;
}

export function getSettings(): Promise<Settings> {
  return invoke<Settings>("get_settings");
}

export function completeWizard(
  language: string,
  grade: number,
  board: string
): Promise<Settings> {
  return invoke<Settings>("complete_wizard", { language, grade, board });
}

export function updateLanguage(language: string): Promise<Settings> {
  return invoke<Settings>("update_language", { language });
}

export interface Subject {
  id: number;
  grade: number;
  name: string;
  nameHi: string;
}

export interface Chapter {
  id: number;
  subjectId: number;
  ord: number;
  title: string;
  titleHi: string;
  summary: string;
}

export interface Concept {
  id: number;
  chapterId: number;
  ord: number;
  name: string;
  keyFormula: string;
  commonError: string;
  examWeight: number;
}

export interface QuestionOption {
  id: number;
  ord: number;
  body: string;
}

export interface Question {
  id: number;
  conceptId: number;
  qtype: string;
  bloomLevel: string;
  difficultyLayer: number;
  prompt: string;
  options: QuestionOption[];
}

export function listSubjects(grade: number): Promise<Subject[]> {
  return invoke<Subject[]>("list_subjects", { grade });
}

export function listChapters(subjectId: number): Promise<Chapter[]> {
  return invoke<Chapter[]>("list_chapters", { subjectId });
}

export function listConcepts(chapterId: number): Promise<Concept[]> {
  return invoke<Concept[]>("list_concepts", { chapterId });
}

export interface AnswerReview {
  questionId: number;
  correct: boolean;
  correctOptionId: number;
  explanation: string;
  trap: string;
}

export interface QuizResult {
  sessionId: number;
  total: number;
  correct: number;
  score: number;
  perQuestion: AnswerReview[];
}

export function startQuiz(
  chapterId: number
): Promise<[number, Question[]]> {
  return invoke<[number, Question[]]>("start_quiz", { chapterId });
}

export function submitAnswer(
  sessionId: number,
  questionId: number,
  chosenOptionId: number,
  day: string
): Promise<AnswerReview> {
  return invoke<AnswerReview>("submit_answer", {
    sessionId,
    questionId,
    chosenOptionId,
    day,
  });
}

/// Local calendar day as `YYYY-MM-DD`, matching the backend's streak day key.
/// Using the local day (not UTC) keeps pre-dawn study in IST attributed to the
/// correct day.
export function localDayString(now: Date = new Date()): string {
  const y = now.getFullYear();
  const m = String(now.getMonth() + 1).padStart(2, "0");
  const d = String(now.getDate()).padStart(2, "0");
  return `${y}-${m}-${d}`;
}

export function finishQuiz(sessionId: number): Promise<QuizResult> {
  return invoke<QuizResult>("finish_quiz", { sessionId });
}

export interface ConceptProgress {
  conceptId: number;
  name: string;
  pKnown: number;
  due: number | null;
  attempts: number;
}

export function getProgress(chapterId: number | null): Promise<ConceptProgress[]> {
  return invoke<ConceptProgress[]>("get_progress", { chapterId });
}

export function getDue(now: number): Promise<ConceptProgress[]> {
  return invoke<ConceptProgress[]>("get_due", { now });
}

export function getStreak(today: string): Promise<number> {
  return invoke<number>("get_streak", { today });
}

export interface GamificationSummary {
  xp: number;
  level: number;
  xpIntoLevel: number;
  xpForNextLevel: number;
  badges: string[];
}

export function getGamification(today: string): Promise<GamificationSummary> {
  return invoke<GamificationSummary>("get_gamification", { today });
}

export interface ChatMessage {
  id: number;
  chapterId: number;
  role: string;
  body: string;
  fsmState: string;
  createdAt: number;
}

export interface TutorTurnResult {
  reply: string;
  fsmState: string;
  studentMsgId: number;
  tutorMsgId: number;
}

export function getChatHistory(chapterId: number): Promise<ChatMessage[]> {
  return invoke<ChatMessage[]>("get_chat_history", { chapterId });
}

export function tutorTurn(
  chapterId: number,
  studentMsg: string,
  fsmState: string
): Promise<TutorTurnResult> {
  return invoke<TutorTurnResult>("tutor_turn", {
    chapterId,
    studentMsg,
    fsmState,
  });
}
