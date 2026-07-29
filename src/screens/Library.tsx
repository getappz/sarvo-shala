import { useEffect, useState, useCallback } from "react";
import { vfsListAll, vfsCreateFolder, type Folder } from "../lib/ipc";
import { buildFolderTree } from "../lib/folderTree";
import FolderTree from "../components/library/FolderTree";

export default function Library({ t }: { t: (k: string) => string }) {
  const [folders, setFolders] = useState<Folder[]>([]);
  const [selectedFolderId, setSelectedFolderId] = useState<number | null>(
    null
  );

  const refetch = useCallback(() => {
    vfsListAll()
      .then(([fs]) => setFolders(fs))
      .catch(() => setFolders([]));
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

  const tree = buildFolderTree(folders);

  return (
    <div className="max-w-5xl mx-auto grid grid-cols-[minmax(180px,280px)_minmax(0,1fr)] gap-4">
      <FolderTree
        tree={tree}
        selectedFolderId={selectedFolderId}
        onSelect={setSelectedFolderId}
        onCreateFolder={handleCreateFolder}
        t={t}
      />
      <div />
    </div>
  );
}
