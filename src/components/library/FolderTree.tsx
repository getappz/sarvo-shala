import type { FolderTreeNode } from "../../lib/folderTree";
import Button from "../ui/Button";

function Node({
  node,
  depth,
  selectedFolderId,
  onSelect,
}: {
  node: FolderTreeNode;
  depth: number;
  selectedFolderId: number | null;
  onSelect: (id: number | null) => void;
}) {
  const selected = node.folder.id === selectedFolderId;
  return (
    <li>
      <button
        onClick={() => onSelect(node.folder.id)}
        style={{ paddingLeft: `${depth * 16 + 12}px` }}
        className={`w-full text-left py-1.5 pr-3 rounded-lg text-sm ${
          selected ? "bg-primary/10 text-primary font-semibold" : "text-ink"
        }`}
      >
        {node.folder.title}
      </button>
      {node.children.length > 0 && (
        <ul className="list-none m-0 p-0">
          {node.children.map((child) => (
            <Node
              key={child.folder.id}
              node={child}
              depth={depth + 1}
              selectedFolderId={selectedFolderId}
              onSelect={onSelect}
            />
          ))}
        </ul>
      )}
    </li>
  );
}

export default function FolderTree({
  tree,
  selectedFolderId,
  onSelect,
  onCreateFolder,
  t,
}: {
  tree: FolderTreeNode[];
  selectedFolderId: number | null;
  onSelect: (id: number | null) => void;
  onCreateFolder: (parentId: number | null) => void;
  t: (k: string) => string;
}) {
  return (
    <div className="flex flex-col gap-2">
      <button
        onClick={() => onSelect(null)}
        className={`w-full text-left py-1.5 px-3 rounded-lg text-sm ${
          selectedFolderId === null
            ? "bg-primary/10 text-primary font-semibold"
            : "text-ink"
        }`}
      >
        {t("library")}
      </button>
      <ul className="list-none m-0 p-0">
        {tree.map((node) => (
          <Node
            key={node.folder.id}
            node={node}
            depth={0}
            selectedFolderId={selectedFolderId}
            onSelect={onSelect}
          />
        ))}
      </ul>
      <Button onClick={() => onCreateFolder(selectedFolderId)}>
        + {t("newFolder")}
      </Button>
    </div>
  );
}
