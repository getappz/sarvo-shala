import Button from "./Button";

export default function ConfirmDialog({
  message,
  onConfirm,
  onCancel,
  t,
}: {
  message: string;
  onConfirm: () => void;
  onCancel: () => void;
  t: (k: string) => string;
}) {
  return (
    <div className="fixed inset-0 bg-ink/40 flex items-center justify-center">
      <div className="bg-surface rounded-xl p-6 max-w-sm w-full">
        <p className="text-sm text-ink mb-4">{message}</p>
        <div className="flex gap-2 justify-end">
          <button
            className="text-sm text-ink-muted underline"
            onClick={onCancel}
          >
            {t("cancel")}
          </button>
          <Button onClick={onConfirm}>{t("delete")}</Button>
        </div>
      </div>
    </div>
  );
}
