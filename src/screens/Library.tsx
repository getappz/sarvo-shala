import { useEffect, useState, useCallback } from "react";
import {
  vfsListAll,
  vfsCreateFolder,
  vfsImportDocument,
  type Folder,
  type FolderItem,
} from "../lib/ipc";
import { buildFolderTree } from "../lib/folderTree";
import FolderTree from "../components/library/FolderTree";
import ContentPane from "../components/library/ContentPane";
import NoteEditor from "../components/library/NoteEditor";

export default function Library({ t }: { t: (k: string) => string }) {
  const [folders, setFolders] = useState<Folder[]>([]);
  const [items, setItems] = useState<FolderItem[]>([]);
  const [selectedFolderId, setSelectedFolderId] = useState<number | null>(
    null
  );
  const [openDocumentId, setOpenDocumentId] = useState<string | null>(null);

  const refetch = useCallback(() => {
    vfsListAll()
      .then(([fs, its]) => {
        setFolders(fs);
        setItems(its);
      })
      .catch(() => {
        setFolders([]);
        setItems([]);
      });
  }, []);

  useEffect(() => {
    refetch();
  }, [refetch]);

  async function handleCreateFolder(parentId: number | null) {
    const title = window.prompt(t("newFolder"));
    if (!title || !title.trim()) return;
    try {
      await vfsCreateFolder(parentId, title.trim());
      refetch();
    } catch {
      // no global error surface yet -- matches this app's existing
      // per-call .catch() convention (see Browse.tsx)
    }
  }

  async function handleCreateNote() {
    const path = `notes/${crypto.randomUUID()}.md`;
    try {
      const doc = await vfsImportDocument(
        selectedFolderId,
        path,
        t("untitledNote"),
        ""
      );
      refetch();
      setOpenDocumentId(doc.id);
    } catch {
      // matches this app's existing per-call .catch() convention
    }
  }

  const tree = buildFolderTree(folders);
  const visibleItems = items.filter((i) => i.folderId === selectedFolderId);

  return (
    <div className="max-w-5xl mx-auto grid grid-cols-[minmax(180px,280px)_minmax(0,1fr)] gap-4">
      <FolderTree
        tree={tree}
        selectedFolderId={selectedFolderId}
        onSelect={setSelectedFolderId}
        onCreateFolder={handleCreateFolder}
        t={t}
      />
      <div>
        {openDocumentId && (
          <button
            className="text-sm text-ink-muted underline mb-2"
            onClick={() => setOpenDocumentId(null)}
          >
            ← {t("library")}
          </button>
        )}
        {openDocumentId ? (
          <NoteEditor
            key={openDocumentId}
            documentId={openDocumentId}
            folderId={selectedFolderId}
            t={t}
          />
        ) : (
          <ContentPane
            items={visibleItems}
            onOpenDocument={setOpenDocumentId}
            onCreateNote={handleCreateNote}
            t={t}
          />
        )}
      </div>
    </div>
  );
}
