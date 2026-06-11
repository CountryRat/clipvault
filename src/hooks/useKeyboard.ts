import { useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { ClipEntry } from "./useClips";

interface UseKeyboardOptions {
  clipsRef: React.MutableRefObject<ClipEntry[]>;
  selectedIdxRef: React.MutableRefObject<number>;
  setSelectedIdx: React.Dispatch<React.SetStateAction<number>>;
  handleCopy: (entry: ClipEntry) => void;
}

export function useKeyboard({
  clipsRef,
  selectedIdxRef,
  setSelectedIdx,
  handleCopy,
}: UseKeyboardOptions) {
  useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      const clips = clipsRef.current;
      const idx = selectedIdxRef.current;

      if (e.key === "Escape") {
        e.preventDefault();
        invoke("hide_window");
      } else if (e.key === "ArrowDown") {
        e.preventDefault();
        setSelectedIdx((p) => Math.min(p + 1, clips.length - 1));
      } else if (e.key === "ArrowUp") {
        e.preventDefault();
        setSelectedIdx((p) => Math.max(p - 1, 0));
      } else if (e.key === "Enter") {
        e.preventDefault();
        if (clips[idx]) {
          handleCopy(clips[idx]);
        }
      }
    };

    window.addEventListener("keydown", onKeyDown, true);
    return () => window.removeEventListener("keydown", onKeyDown, true);
  }, [clipsRef, selectedIdxRef, setSelectedIdx, handleCopy]);
}
