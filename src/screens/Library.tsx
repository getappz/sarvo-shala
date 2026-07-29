import { useEffect, useMemo, useState, useCallback } from "react";
import {
  vfsListAll,
  vfsListDocuments,
  vfsCreateFolder,
  vfsImportDocument,
  vfsDeleteFolder,
  type Folder,
  type FolderItem,
  type VfsDocumentSummary,
} from "../lib/ipc";
import { buildFolderTree } from "../lib/folderTree";
import FolderTree from "../components/library/FolderTree";
import ContentPane from "../components/library/ContentPane";
import NoteEditor from "../components/library/NoteEditor";
import ConfirmDialog from "../components/ui/ConfirmDialog";

export default function Library({ t }: { t: (k: string) => string }) {
  const [folders, setFolders] = useState<Folder[]>([]);
  const [items, setItems] = useState<FolderItem[]>([]);
  const [documents, setDocuments] = useState<VfsDocumentSummary[]>([]);
  const [selectedFolderId, setSelectedFolderId] = useState<number | null>(
    null
  );
  const [openDocumentId, setOpenDocumentId] = useState<string | null>(null);
  const [pendingDeleteFolderId, setPendingDeleteFolderId] = useState<
    number | null
  >(null);

  const refetch = useCallback(() => {
    Promise.all([vfsListAll(), vfsListDocuments()])
      .then(([[fs, its], docs]) => {
        setFolders(fs);
        setItems(its);
        setDocuments(docs);
      })
      .catch(() => {
        setFolders([]);
        setItems([]);
        setDocuments([]);
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

  function requestDeleteFolder(folderId: number) {
    setPendingDeleteFolderId(folderId);
  }

  async function confirmDeleteFolder() {
    if (pendingDeleteFolderId == null) return;
    try {
      await vfsDeleteFolder(pendingDeleteFolderId);
      if (selectedFolderId === pendingDeleteFolderId) {
        setSelectedFolderId(null);
      }
      refetch();
    } catch {
      // matches this app's existing per-call .catch() convention
    } finally {
      setPendingDeleteFolderId(null);
    }
  }

  const tree = buildFolderTree(folders);
  const visibleItems = items.filter((i) => i.folderId === selectedFolderId);
  const docTitles = useMemo(
    () => new Map(documents.map((d) => [d.id, d.title])),
    [documents]
  );

  return (
    <>
      <div className="max-w-5xl mx-auto grid grid-cols-[minmax(180px,280px)_minmax(0,1fr)] gap-4">
        <FolderTree
          tree={tree}
          selectedFolderId={selectedFolderId}
          onSelect={setSelectedFolderId}
          onCreateFolder={handleCreateFolder}
          onDeleteFolder={requestDeleteFolder}
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
              docTitles={docTitles}
              onOpenDocument={setOpenDocumentId}
              onCreateNote={handleCreateNote}
              t={t}
            />
          )}
        </div>
      </div>
      {pendingDeleteFolderId != null && (
        <ConfirmDialog
          message={t("deleteFolderConfirm")}
          onConfirm={confirmDeleteFolder}
          onCancel={() => setPendingDeleteFolderId(null)}
          t={t}
        />
      )}
    </>
  );
}
