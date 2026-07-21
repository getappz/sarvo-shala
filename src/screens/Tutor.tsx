import { useEffect, useRef, useState } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import {
  getChatHistory,
  tutorTurn,
  type ChatMessage,
} from "../lib/ipc";
import type { Chapter } from "../lib/ipc";
import Button from "../components/ui/Button";

type TFn = (k: string) => string;

const STATE_KEY: Record<string, string> = {
  Probe: "stateProbe",
  Clarify: "stateClarify",
  Confront: "stateConfront",
  Scaffold: "stateScaffold",
  Confirm: "stateConfirm",
};

// The persisted FSM string may carry a scaffold-turn counter ("Scaffold:3");
// strip it for display/label lookup.
const baseState = (fsm: string) => fsm.split(":")[0];
const stateLabel = (fsm: string, t: TFn) =>
  t(STATE_KEY[baseState(fsm)] ?? "stateProbe");

export default function Tutor({
  chapter,
  t,
}: {
  chapter: Chapter;
  t: TFn;
}) {
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [input, setInput] = useState("");
  const [fsm, setFsm] = useState("Probe");
  const [busy, setBusy] = useState(false);

  const fsmRef = useRef(fsm);
  fsmRef.current = fsm;
  const streamingId = useRef<number>(-1);
  // Monotonic counter for synthetic message ids so the optimistic student
  // bubble and the streamed tutor stub can never collide as React keys.
  const synthId = useRef(-1);
  const nextSynthId = () => {
    synthId.current -= 1;
    return synthId.current;
  };

  useEffect(() => {
    let cancelled = false;
    setMessages([]);
    setFsm("Probe");
    streamingId.current = -1;
    getChatHistory(chapter.id)
      .then((hist) => {
        if (cancelled) return;
        setMessages(hist);
        // Only open the probe once; the cancelled flag also guards React 18
        // StrictMode's double-invoked effect, which would otherwise persist
        // two opening turns.
        if (hist.length === 0 && !busy) {
          doSend("");
        }
      })
      .catch(() => {
        if (!cancelled) setMessages([]);
      });
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [chapter.id]);

  useEffect(() => {
    let unlisten: UnlistenFn | undefined;
    const p = listen<{ text?: string; state?: string; chapterId?: number }>(
      "tutor-stream",
      (e) => {
        const payload = e.payload;
        // Cross-chapter guard: a token/state from a previous chapter's stream
        // must not bleed into the current chapter's chat.
        if (
          payload.chapterId !== undefined &&
          payload.chapterId !== chapter.id
        ) {
          return;
        }
        if (payload.text !== undefined) {
          setMessages((prev) => {
            const copy = [...prev];
            const last = copy[copy.length - 1];
            if (
              last &&
              last.role === "tutor" &&
              last.id === streamingId.current
            ) {
              copy[copy.length - 1] = {
                ...last,
                body: last.body + payload.text,
              };
            } else {
              const id = nextSynthId();
              streamingId.current = id;
              copy.push({
                id,
                chapterId: chapter.id,
                role: "tutor",
                body: payload.text ?? "",
                fsmState: fsmRef.current,
                createdAt: Date.now(),
              });
            }
            return copy;
          });
        } else if (payload.state !== undefined) {
          setFsm(payload.state);
        }
      }
    );
    p.then((u) => (unlisten = u));
    return () => {
      if (unlisten) unlisten();
    };
  }, [chapter.id]);

  async function doSend(text: string) {
    if (busy) return;
    setBusy(true);
    const trimmed = text.trim();
    if (trimmed.length > 0) {
      const studentMsg: ChatMessage = {
        id: nextSynthId(),
        chapterId: chapter.id,
        role: "student",
        body: trimmed,
        fsmState: fsm,
        createdAt: Date.now(),
      };
      setMessages((m) => [...m, studentMsg]);
    }
    setInput("");
    streamingId.current = -1;
    try {
      const res = await tutorTurn(chapter.id, trimmed, fsm);
      setFsm(res.fsmState);
      setMessages((prev) => {
        const copy = [...prev];
        const li = copy.length - 1;
        if (
          copy[li] &&
          copy[li].role === "tutor" &&
          copy[li].id === streamingId.current
        ) {
          copy[li] = {
            ...copy[li],
            body: res.reply,
            fsmState: res.fsmState,
          };
        } else {
          copy.push({
            id: res.tutorMsgId,
            chapterId: chapter.id,
            role: "tutor",
            body: res.reply,
            fsmState: res.fsmState,
            createdAt: Date.now(),
          });
        }
        return copy;
      });
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="flex flex-col h-full max-w-2xl mx-auto">
      <h2 className="font-display text-lg mb-1">
        {t("tutor")} — {chapter.titleHi || chapter.title}
      </h2>
      <div className="inline-block w-fit text-xs font-semibold text-primary bg-primary/10 px-2.5 py-1 rounded-full mb-3">
        {stateLabel(fsm, t)}
      </div>

      <div
        className="flex-1 overflow-auto flex flex-col gap-2.5 mb-3"
        role="log"
        aria-live="polite"
      >
        {messages.map((m) => (
          <div
            key={m.id}
            className={
              m.role === "student"
                ? "max-w-[75%] self-end rounded-xl px-3.5 py-2.5 text-sm bg-primary text-white"
                : "max-w-[75%] self-start rounded-xl px-3.5 py-2.5 text-sm bg-border/60 text-ink"
            }
          >
            <span className="block text-[11px] font-semibold mb-0.5 opacity-70">
              {m.role === "student" ? t("you") : t("tutor")}
            </span>
            <p>{m.body}</p>
          </div>
        ))}
      </div>

      <form
        className="flex gap-2"
        onSubmit={(e) => {
          e.preventDefault();
          doSend(input);
        }}
      >
        <input
          value={input}
          onChange={(e) => setInput(e.target.value)}
          placeholder={t("tutorPlaceholder")}
          disabled={busy}
          className="flex-1 border border-border rounded-lg px-3 py-2 text-sm font-sans focus:outline-none focus:ring-2 focus:ring-primary/40"
        />
        <Button type="submit" disabled={busy || input.trim().length === 0}>
          {t("send")}
        </Button>
      </form>
    </div>
  );
}
