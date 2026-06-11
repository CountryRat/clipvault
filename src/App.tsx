import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useClips } from "./hooks/useClips";
import { useKeyboard } from "./hooks/useKeyboard";
import { SearchBar } from "./components/SearchBar";
import { ClipList } from "./components/ClipList";
import { SettingsPanel } from "./components/SettingsPanel";
import { StatusFooter } from "./components/StatusFooter";

export default function App() {
  const [query, setQuery] = useState("");
  const [pinnedOnly, setPinnedOnly] = useState(false);
  const [showSettings, setShowSettings] = useState(false);
  const [bgColor, setBgColor] = useState("#0f0f0f");
  const [opacity, setOpacity] = useState(1.0);

  // First-launch autostart prompt
  const [showAutostartPrompt, setShowAutostartPrompt] = useState(false);

  const {
    clips,
    selectedIdx,
    setSelectedIdx,
    clipsRef,
    selectedIdxRef,
    handleCopy,
    handleTogglePin,
    handleDelete,
  } = useClips({ query, pinnedOnly });

  useKeyboard({
    clipsRef,
    selectedIdxRef,
    setSelectedIdx,
    handleCopy,
  });

  // Load config + check if we should prompt for autostart
  useEffect(() => {
    invoke<{
      bg_color: string;
      opacity: number;
      asked_autostart: boolean;
    }>("get_config")
      .then((cfg) => {
        if (cfg.bg_color) {
          setBgColor(cfg.bg_color);
          document.documentElement.style.setProperty("--bg-primary", cfg.bg_color);
        }
        if (cfg.opacity != null) setOpacity(cfg.opacity);
        if (!cfg.asked_autostart) setShowAutostartPrompt(true);
      })
      .catch(() => {});
  }, []);

  const handleAutostartAnswer = async (enable: boolean) => {
    if (enable) {
      await invoke("toggle_autostart").catch(() => {});
    }
    await invoke("mark_autostart_asked").catch(() => {});
    setShowAutostartPrompt(false);
  };

  if (showSettings) {
    return <SettingsPanel onClose={() => setShowSettings(false)} />;
  }

  // First-launch prompt overlay
  if (showAutostartPrompt) {
    return (
      <div className="flex flex-col items-center justify-center h-screen gap-4 px-6" style={{ backgroundColor: bgColor }}>
        <p className="text-sm text-[var(--text-primary)] text-center">
          是否开机自启 ClipVault？
        </p>
        <p className="text-[10px] text-[var(--text-secondary)] text-center">
          可随时在设置中更改
        </p>
        <div className="flex items-center gap-3 mt-2">
          <button
            className="text-xs px-4 py-1.5 rounded bg-[var(--accent)] text-white hover:bg-[var(--accent-dim)] transition-colors"
            onClick={() => handleAutostartAnswer(true)}
          >
            开启
          </button>
          <button
            className="text-xs px-4 py-1.5 rounded border border-[var(--border)] text-[var(--text-secondary)] hover:text-[var(--text-primary)] transition-colors"
            onClick={() => handleAutostartAnswer(false)}
          >
            暂不
          </button>
        </div>
      </div>
    );
  }

  return (
    <div
      className="flex flex-col h-screen"
      style={{ backgroundColor: bgColor, opacity: opacity }}
    >
      <SearchBar
        query={query}
        onQueryChange={setQuery}
        pinnedOnly={pinnedOnly}
        onPinnedToggle={() => setPinnedOnly((p) => !p)}
        onSettingsClick={() => setShowSettings(true)}
      />
      <ClipList
        clips={clips}
        query={query}
        selectedIdx={selectedIdx}
        onSelectIdx={setSelectedIdx}
        onCopy={handleCopy}
        onTogglePin={handleTogglePin}
        onDelete={handleDelete}
      />
      <StatusFooter clipCount={clips.length} />
    </div>
  );
}
