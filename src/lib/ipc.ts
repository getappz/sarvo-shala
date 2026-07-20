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
  chosenOptionId: number
): Promise<AnswerReview> {
  return invoke<AnswerReview>("submit_answer", {
    sessionId,
    questionId,
    chosenOptionId,
  });
}

export function finishQuiz(sessionId: number): Promise<QuizResult> {
  return invoke<QuizResult>("finish_quiz", { sessionId });
}
