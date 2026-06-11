interface StatusFooterProps {
  clipCount: number;
}

export function StatusFooter({ clipCount }: StatusFooterProps) {
  return (
    <div className="flex items-center justify-between px-4 py-2 border-t border-[var(--border)] text-[10px] text-[var(--text-secondary)]">
      <span>{clipCount} 条记录 · ↑↓ 导航 · Enter 粘贴 · Esc 隐藏</span>
      <span>Ctrl+Shift+V 切换面板</span>
    </div>
  );
}
