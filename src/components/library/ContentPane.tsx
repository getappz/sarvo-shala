import type { FolderItem } from "../../lib/ipc";
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
  return (
    <div className="flex flex-col gap-3">
      <div className="flex gap-2">
        <Button onClick={onCreateNote}>+ {t("newNote")}</Button>
      </div>

      {items.length === 0 ? (
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
