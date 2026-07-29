import { useEffect, useState } from "react";
import {
  vfsSearchDocuments,
  type FolderItem,
  type VfsDocumentMatch,
} from "../../lib/ipc";
import Button from "../ui/Button";

export default function ContentPane({
  items,
  onOpenDocument,
  onCreateNote,
  t,
}: {
  items: FolderItem[];
  onOpenDocument: (itemId: string) => void;
  onCreateNote: () => void;
  t: (k: string) => string;
}) {
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<VfsDocumentMatch[] | null>(null);

  useEffect(() => {
    if (!query.trim()) {
      setResults(null);
      return;
    }
    const handle = setTimeout(() => {
      vfsSearchDocuments(query.trim(), 20)
        .then(setResults)
        .catch(() => setResults([]));
    }, 300);
    return () => clearTimeout(handle);
  }, [query]);

  return (
    <div className="flex flex-col gap-3">
      <input
        type="search"
        value={query}
        onChange={(e) => setQuery(e.target.value)}
        placeholder={t("searchPlaceholder")}
        className="px-3 py-2 rounded-lg border border-border bg-surface text-sm"
      />

      <div className="flex gap-2">
        <Button onClick={onCreateNote}>+ {t("newNote")}</Button>
      </div>

      {results !== null ? (
        results.length === 0 ? (
          <p className="text-sm text-ink-muted">{t("noItemsInFolder")}</p>
        ) : (
          <ul className="list-none m-0 p-0 flex flex-col gap-1.5">
            {results.map((r) => (
              <li key={r.id}>
                <button
                  onClick={() => onOpenDocument(r.id)}
                  className="w-full text-left px-3 py-2 rounded-lg border border-border bg-surface text-sm"
                >
                  <div className="font-semibold">{r.path}</div>
                  <div className="text-xs text-ink-muted">{r.snippet}</div>
                </button>
              </li>
            ))}
          </ul>
        )
      ) : items.length === 0 ? (
        <p className="text-sm text-ink-muted">{t("noItemsInFolder")}</p>
      ) : (
        <ul className="list-none m-0 p-0 flex flex-col gap-1.5">
          {items.map((item) => (
            <li key={item.id}>
              <button
                onClick={() => onOpenDocument(item.itemId)}
                className="w-full text-left px-3 py-2 rounded-lg border border-border bg-surface text-sm"
              >
                📄 {item.itemId}
              </button>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
