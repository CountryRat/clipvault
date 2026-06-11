import { useRef, useEffect, useCallback } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";

interface SearchBarProps {
  query: string;
  onQueryChange: (q: string) => void;
  pinnedOnly: boolean;
  onPinnedToggle: () => void;
  onSettingsClick: () => void;
}

export function SearchBar({
  query,
  onQueryChange,
  pinnedOnly,
  onPinnedToggle,
  onSettingsClick,
}: SearchBarProps) {
  const inputRef = useRef<HTMLInputElement>(null);
  const dragRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  // Drag the window via the search bar (excludes input/buttons)
  const handleMouseDown = useCallback((e: React.MouseEvent) => {
    // Only start drag on non-interactive elements (the search bar itself)
    const target = e.target as HTMLElement;
    // Don't start drag on input or buttons
    if (target.tagName === "INPUT" || target.closest("button")) {
      return;
    }
    getCurrentWindow().startDragging();
  }, []);

  return (
    <div
      ref={dragRef}
      onMouseDown={handleMouseDown}
      className="flex items-center gap-2 px-4 py-3 border-b border-[var(--border)] cursor-grab active:cursor-grabbing"
    >
      <svg
        className="w-4 h-4 text-[var(--text-secondary)] shrink-0"
        fill="none"
        stroke="currentColor"
        viewBox="0 0 24 24"
      >
        <path
          strokeLinecap="round"
          strokeLinejoin="round"
          strokeWidth={2}
          d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
        />
      </svg>
      <input
        ref={inputRef}
        type="text"
        className="flex-1 bg-transparent text-sm text-[var(--text-primary)] placeholder-[var(--text-secondary)]"
        placeholder="搜索剪贴板历史..."
        value={query}
        onChange={(e) => onQueryChange(e.target.value)}
      />
      <button
        className={`text-xs px-2 py-1 rounded transition-colors ${
          pinnedOnly
            ? "bg-[var(--tag-bg)] text-[var(--accent)]"
            : "text-[var(--text-secondary)] hover:text-[var(--text-primary)]"
        }`}
        onClick={onPinnedToggle}
      >
        ⭐ 收藏
      </button>
      <button
        className="p-1 text-[var(--text-secondary)] hover:text-[var(--text-primary)] transition-colors"
        title="设置"
        onClick={onSettingsClick}
      >
        <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={2}
            d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.066 2.573c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.573 1.066c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.066-2.573c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"
          />
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={2}
            d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
          />
        </svg>
      </button>
    </div>
  );
}
