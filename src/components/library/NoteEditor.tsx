import { useEffect, useRef, useState } from "react";
import { Crepe, CrepeFeature } from "@milkdown/crepe";
import "@milkdown/crepe/theme/common/style.css";
import "@milkdown/crepe/theme/frame.css";
import {
  vfsGetDocument,
  vfsImportDocument,
  type VfsDocument,
} from "../../lib/ipc";

const AUTOSAVE_DELAY_MS = 800;

export default function NoteEditor({
  documentId,
  folderId,
  t,
}: {
  documentId: string;
  folderId: number | null;
  t: (k: string) => string;
}) {
  const containerRef = useRef<HTMLDivElement>(null);
  const crepeRef = useRef<Crepe | null>(null);
  const docRef = useRef<VfsDocument | null>(null);
  const saveTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const [doc, setDoc] = useState<VfsDocument | null>(null);
  const [saveFailed, setSaveFailed] = useState(false);

  useEffect(() => {
    let cancelled = false;
    vfsGetDocument(documentId).then((d) => {
      if (!cancelled) setDoc(d);
    });
    return () => {
      cancelled = true;
    };
  }, [documentId]);

  useEffect(() => {
    if (!doc || !containerRef.current) return;
    docRef.current = doc;

    const crepe = new Crepe({
      root: containerRef.current,
      defaultValue: doc.content,
      features: {
        [CrepeFeature.ListItem]: true,
        [CrepeFeature.LinkTooltip]: true,
        [CrepeFeature.Cursor]: true,
        [CrepeFeature.BlockEdit]: true,
        [CrepeFeature.Toolbar]: true,
        [CrepeFeature.Placeholder]: true,
        [CrepeFeature.Table]: true,
        [CrepeFeature.CodeMirror]: false,
        [CrepeFeature.ImageBlock]: false,
        [CrepeFeature.Latex]: false,
      },
    });
    crepeRef.current = crepe;

    function scheduleSave() {
      if (saveTimer.current) clearTimeout(saveTimer.current);
      saveTimer.current = setTimeout(async () => {
        const current = docRef.current;
        if (!current || !crepeRef.current) return;
        try {
          const content = crepeRef.current.getMarkdown();
          const saved = await vfsImportDocument(
            folderId,
            current.path,
            current.title,
            content
          );
          docRef.current = saved;
          setSaveFailed(false);
        } catch {
          setSaveFailed(true);
        }
      }, AUTOSAVE_DELAY_MS);
    }

    const container = containerRef.current;
    container.addEventListener("input", scheduleSave);

    crepe.create();

    return () => {
      container.removeEventListener("input", scheduleSave);
      if (saveTimer.current) clearTimeout(saveTimer.current);
      crepe.destroy();
      crepeRef.current = null;
    };
    // remounted via React key={documentId} on note switch
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [doc]);

  if (!doc) return null;

  return (
    <div>
      {saveFailed && (
        <p className="text-xs text-amber-800 bg-amber-50 px-2 py-1 rounded mb-2">
          {t("failedToSave")}
        </p>
      )}
      <div ref={containerRef} />
    </div>
  );
}
