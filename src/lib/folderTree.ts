import type { Folder } from "./ipc";

export interface FolderTreeNode {
  folder: Folder;
  children: FolderTreeNode[];
}

export function buildFolderTree(folders: Folder[]): FolderTreeNode[] {
  const byParent = new Map<number | null, Folder[]>();
  for (const f of folders) {
    const siblings = byParent.get(f.parentId) ?? [];
    siblings.push(f);
    byParent.set(f.parentId, siblings);
  }

  const sortSiblings = (list: Folder[]) =>
    list.slice().sort((a, b) => a.sortOrder - b.sortOrder || a.id - b.id);

  function build(parentId: number | null): FolderTreeNode[] {
    return sortSiblings(byParent.get(parentId) ?? []).map((folder) => ({
      folder,
      children: build(folder.id),
    }));
  }

  return build(null);
}
