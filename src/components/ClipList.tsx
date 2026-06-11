import { useRef, useEffect } from "react";
import type { ClipEntry } from "../hooks/useClips";

function timeAgo(iso: string): string {
  const now = Date.now();
  const then = Date.parse(iso);
  const secs = Math.floor((now - then) / 1000);
  if (secs < 60) return "刚刚";
  if (secs < 3600) return `${Math.floor(secs / 60)} 分钟前`;
  if (secs < 86400) return `${Math.floor(secs / 3600)} 小时前`;
  return `${Math.floor(secs / 86400)} 天前`;
}

function HighlightedText({ text, query }: { text: string; query: string }) {
  if (!query) return <>{text.slice(0, 200)}</>;
  const idx = text.toLowerCase().indexOf(query.toLowerCase());
  if (idx === -1) return <>{text.slice(0, 200)}</>;

  const before = text.slice(Math.max(0, idx - 30), idx);
  const match = text.slice(idx, idx + query.length);
  const after = text.slice(idx + query.length, idx + query.length + 170);

  return (
    <>
      {idx > 30 && "…"}
      {before}
      <mark className="bg-[var(--accent)]/30 text-[var(--accent)] rounded px-0.5">
        {match}
      </mark>
      {after}
    </>
  );
}

interface ClipListProps {
  clips: ClipEntry[];
  query: string;
  selectedIdx: number;
  onSelectIdx: (idx: number) => void;
  onCopy: (clip: ClipEntry) => void;
  onTogglePin: (id: number) => void;
  onDelete: (id: number) => void;
}

export function ClipList({
  clips,
  query,
  selectedIdx,
  onSelectIdx,
  onCopy,
  onTogglePin,
  onDelete,
}: ClipListProps) {
  const listRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const el = listRef.current?.children[selectedIdx] as HTMLElement | undefined;
    el?.scrollIntoView({ block: "nearest" });
  }, [selectedIdx]);

  if (clips.length === 0) {
    return (
      <div className="flex-1 flex flex-col items-center justify-center text-[var(--text-secondary)] gap-2">
        <svg
          className="w-10 h-10 opacity-30"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={1}
            d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2"
          />
        </svg>
        <p className="text-xs">
          {query ? "没有匹配的记录" : "剪贴板为空，开始复制吧！"}
        </p>
      </div>
    );
  }

  return (
    <div ref={listRef} className="flex-1 overflow-y-auto">
      {clips.map((clip, idx) => (
        <div
          key={clip.id}
          className={`group flex items-start gap-3 px-4 py-2.5 border-b border-[var(--border)]/50 cursor-pointer transition-colors ${
            idx === selectedIdx
              ? "bg-[var(--bg-hover)]"
              : "hover:bg-[var(--bg-secondary)]"
          }`}
          onClick={() => onCopy(clip)}
          onMouseEnter={() => onSelectIdx(idx)}
        >
          <div className="flex-1 min-w-0">
            <p className="text-sm text-[var(--text-primary)] truncate leading-relaxed">
              <HighlightedText text={clip.content} query={query} />
            </p>
            <div className="flex items-center gap-2 mt-1">
              <span className="text-[10px] text-[var(--text-secondary)]">
                {timeAgo(clip.created_at)}
              </span>
              {clip.is_pinned && (
                <span className="text-[10px] text-[var(--pin-color)]">⭐</span>
              )}
              <span className="text-[10px] text-[var(--text-secondary)] uppercase">
                {clip.content_type}
              </span>
            </div>
          </div>

          <div className="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity shrink-0">
            <button
              className="p-1 rounded hover:bg-[var(--bg-hover)] text-[var(--text-secondary)] hover:text-[var(--pin-color)] transition-colors"
              title={clip.is_pinned ? "取消收藏" : "收藏"}
              onClick={(e) => {
                e.stopPropagation();
                onTogglePin(clip.id);
              }}
            >
              <svg
                className="w-3.5 h-3.5"
                fill={clip.is_pinned ? "currentColor" : "none"}
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M5 5a2 2 0 012-2h10a2 2 0 012 2v16l-7-3.5L5 21V5z"
                />
              </svg>
            </button>
            <button
              className="p-1 rounded hover:bg-[var(--bg-hover)] text-[var(--text-secondary)] hover:text-red-400 transition-colors"
              title="删除"
              onClick={(e) => {
                e.stopPropagation();
                onDelete(clip.id);
              }}
            >
              <svg
                className="w-3.5 h-3.5"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"
                />
              </svg>
            </button>
          </div>
        </div>
      ))}
    </div>
  );
}
