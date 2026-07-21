import { getCurrentWindow } from "@tauri-apps/api/window";

export default function Titlebar() {
  const win = getCurrentWindow();

  return (
    <header
      data-tauri-drag-region
      className="flex items-center justify-between h-9 px-3 bg-primary text-white select-none"
    >
      <span
        data-tauri-drag-region
        className="font-display text-sm tracking-wide"
      >
        Sarvo Shala
      </span>
      <div className="flex items-center gap-1">
        <button
          aria-label="Minimize"
          onClick={() => win.minimize()}
          className="w-7 h-7 rounded hover:bg-white/15 flex items-center justify-center"
        >
          &#8211;
        </button>
        <button
          aria-label="Maximize"
          onClick={() => win.toggleMaximize()}
          className="w-7 h-7 rounded hover:bg-white/15 flex items-center justify-center"
        >
          &#9633;
        </button>
        <button
          aria-label="Close"
          onClick={() => win.close()}
          className="w-7 h-7 rounded hover:bg-red-500 flex items-center justify-center"
        >
          &#10005;
        </button>
      </div>
    </header>
  );
}
