import { useState, useEffect, useRef, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";

interface SettingsPanelProps {
  onClose: () => void;
}

function eventToShortcut(e: KeyboardEvent): string | null {
  const { ctrlKey, shiftKey, altKey, metaKey, code } = e;
  if (["ControlLeft", "ControlRight", "ShiftLeft", "ShiftRight",
       "AltLeft", "AltRight", "MetaLeft", "MetaRight"].includes(code)) {
    return null;
  }
  let keyName: string;
  if (code.startsWith("Key")) keyName = code.slice(3);
  else if (code.startsWith("Digit")) keyName = code.slice(5);
  else keyName = code;
  const parts: string[] = [];
  if (ctrlKey) parts.push("Ctrl");
  if (shiftKey) parts.push("Shift");
  if (altKey) parts.push("Alt");
  if (metaKey) parts.push("Super");
  parts.push(keyName);
  return parts.join("+");
}

export function SettingsPanel({ onClose }: SettingsPanelProps) {
  const [shortcut, setShortcut] = useState("");
  const [status, setStatus] = useState<"idle" | "recording" | "saved" | "error">("idle");
  const [statusMsg, setStatusMsg] = useState("");
  const [bgColor, setBgColor] = useState("#0f0f0f");
  const [opacity, setOpacity] = useState(1.0);
  const [autostart, setAutostart] = useState(false);
  const [appearStatus, setAppearStatus] = useState("");
  const inputRef = useRef<HTMLInputElement>(null);
  const debounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  // Load config
  useEffect(() => {
    invoke<{ shortcut: string; bg_color: string; opacity: number; autostart: boolean }>("get_config")
      .then((cfg) => {
        setShortcut(cfg.shortcut);
        setBgColor(cfg.bg_color ?? "#0f0f0f");
        setOpacity(cfg.opacity ?? 1.0);
        setAutostart(cfg.autostart ?? false);
      })
      .catch(() => setShortcut("Ctrl+Shift+V"));
  }, []);

  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  // --- Shortcut recording ---
  const handleFocus = useCallback(() => {
    setStatus("recording");
    setStatusMsg("按下组合键...");
  }, []);

  const handleKeyDown = useCallback((e: React.KeyboardEvent<HTMLInputElement>) => {
    e.preventDefault();
    const shortcutStr = eventToShortcut(e.nativeEvent);
    if (!shortcutStr) return;
    setShortcut(shortcutStr);
    setStatus("idle"); setStatusMsg("");
    invoke<string>("update_shortcut", { shortcutStr })
      .then(() => {
        setStatus("saved"); setStatusMsg("已保存");
        setTimeout(() => { setStatus("idle"); setStatusMsg(""); }, 1500);
      })
      .catch((err) => { setStatus("error"); setStatusMsg(String(err)); });
  }, []);

  const handleBlur = useCallback(() => {
    if (status === "recording") { setStatus("idle"); setStatusMsg(""); }
  }, [status]);

  // --- Appearance: debounced save ---
  const saveAppearance = useCallback((color: string, op: number) => {
    if (debounceRef.current) clearTimeout(debounceRef.current);
    debounceRef.current = setTimeout(() => {
      setAppearStatus("保存中...");
      invoke("update_appearance", { bgColor: color, opacity: op })
        .then(() => {
          setAppearStatus("已保存");
          // Apply to CSS variables immediately
          document.documentElement.style.setProperty("--bg-primary", color);
          document.documentElement.style.setProperty("--bg-overlay", `${color}${Math.round(op * 255).toString(16).padStart(2, "0")}`);
          setTimeout(() => setAppearStatus(""), 1500);
        })
        .catch((err) => setAppearStatus(`错误: ${err}`));
    }, 300);
  }, []);

  const handleColorChange = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
    const color = e.target.value;
    setBgColor(color);
    saveAppearance(color, opacity);
  }, [opacity, saveAppearance]);

  const handleOpacityChange = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
    const op = parseFloat(e.target.value);
    setOpacity(op);
    saveAppearance(bgColor, op);
  }, [bgColor, saveAppearance]);

  return (
    <div className="absolute inset-0 z-10 flex flex-col bg-[var(--bg-primary)]">
      <div className="flex items-center justify-between px-4 py-3 border-b border-[var(--border)]">
        <h2 className="text-sm font-medium text-[var(--text-primary)]">设置</h2>
        <button
          className="p-1 text-[var(--text-secondary)] hover:text-[var(--text-primary)] transition-colors"
          onClick={onClose}
        >
          <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <div className="flex-1 px-4 py-6 space-y-6 overflow-y-auto">
        {/* Hotkey */}
        <div>
          <label className="block text-xs text-[var(--text-secondary)] mb-2">全局快捷键</label>
          <div className="relative">
            <input
              ref={inputRef}
              type="text"
              readOnly
              className={`w-full bg-[var(--bg-secondary)] border ${
                status === "error" ? "border-red-500"
                : status === "saved" ? "border-green-500"
                : "border-[var(--border)]"
              } rounded px-3 py-2 text-sm text-[var(--text-primary)] text-center transition-colors`}
              value={shortcut}
              placeholder="Ctrl+Shift+V"
              onFocus={handleFocus}
              onKeyDown={handleKeyDown}
              onBlur={handleBlur}
            />
            {statusMsg && (
              <span className={`absolute right-3 top-1/2 -translate-y-1/2 text-[10px] ${
                status === "error" ? "text-red-400"
                : status === "saved" ? "text-green-400"
                : "text-[var(--text-secondary)]"
              }`}>{statusMsg}</span>
            )}
          </div>
          <p className="mt-1 text-[10px] text-[var(--text-secondary)]">点击输入框，然后按下组合键</p>
        </div>

        {/* Appearance */}
        <div>
          <label className="block text-xs text-[var(--text-secondary)] mb-3">外观</label>

          <div className="flex items-center gap-3 mb-3">
            <label className="text-[10px] text-[var(--text-secondary)] shrink-0">背景色</label>
            <input
              type="color"
              className="w-8 h-8 rounded cursor-pointer border-0 p-0"
              value={bgColor}
              onChange={handleColorChange}
            />
            <code className="text-[10px] text-[var(--text-secondary)]">{bgColor}</code>
          </div>

          <div className="flex items-center gap-3">
            <label className="text-[10px] text-[var(--text-secondary)] shrink-0">
              不透明度 {Math.round(opacity * 100)}%
            </label>
            <input
              type="range"
              min="0"
              max="1"
              step="0.05"
              className="flex-1 accent-[var(--accent)]"
              value={opacity}
              onChange={handleOpacityChange}
            />
          </div>

          {appearStatus && (
            <span className={`text-[10px] mt-2 inline-block ${
              appearStatus.startsWith("错误") ? "text-red-400" : "text-green-400"
            }`}>{appearStatus}</span>
          )}
        </div>

        {/* Autostart */}
        <div>
          <label className="flex items-center justify-between cursor-pointer">
            <span className="text-xs text-[var(--text-secondary)]">开机自启</span>
            <button
              className={`relative w-9 h-5 rounded-full transition-colors ${
                autostart ? "bg-[var(--accent)]" : "bg-[var(--border)]"
              }`}
              onClick={async () => {
                try {
                  const enabled = await invoke<boolean>("toggle_autostart");
                  setAutostart(enabled);
                } catch (_) {}
              }}
            >
              <span
                className={`absolute top-0.5 w-4 h-4 rounded-full bg-white transition-transform ${
                  autostart ? "translate-x-4" : "translate-x-0.5"
                }`}
              />
            </button>
          </label>
        </div>
      </div>
    </div>
  );
}
