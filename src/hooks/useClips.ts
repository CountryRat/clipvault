import { useState, useEffect, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";

export interface ClipEntry {
  id: number;
  content: string;
  content_hash: string;
  content_type: string;
  source_app: string | null;
  is_pinned: boolean;
  tags: string | null;
  created_at: string;
}

interface UseClipsOptions {
  query: string;
  pinnedOnly: boolean;
  limit?: number;
}

export function useClips({ query, pinnedOnly, limit = 100 }: UseClipsOptions) {
  const [clips, setClips] = useState<ClipEntry[]>([]);
  const [selectedIdx, setSelectedIdx] = useState(0);
  const clipsRef = useRef<ClipEntry[]>([]);
  clipsRef.current = clips;
  const selectedIdxRef = useRef(0);
  selectedIdxRef.current = selectedIdx;

  const fetchClips = useCallback(async () => {
    try {
      const result = await invoke<ClipEntry[]>("search_clips", {
        query,
        limit,
        offset: 0,
        pinnedOnly,
      });
      setClips(result);
    } catch (e) {
      console.error("搜索失败:", e);
    }
  }, [query, pinnedOnly, limit]);

  // Fetch when deps change
  useEffect(() => {
    fetchClips();
  }, [fetchClips]);

  // Poll clipboard every 800ms
  const fetchClipsRef = useRef(fetchClips);
  fetchClipsRef.current = fetchClips;

  useEffect(() => {
    const interval = setInterval(async () => {
      try {
        const result = await invoke<ClipEntry | null>("check_clipboard");
        if (result) {
          fetchClipsRef.current();
        }
      } catch (_) {
        // ignore clipboard read errors
      }
    }, 800);
    return () => clearInterval(interval);
  }, []);

  // Reset selection when list length changes
  useEffect(() => {
    setSelectedIdx(0);
  }, [clips.length]);

  const handleCopy = useCallback(async (entry: ClipEntry) => {
    try {
      await navigator.clipboard.writeText(entry.content);
      fetchClipsRef.current();
    } catch (e) {
      console.error("复制失败:", e);
    }
  }, []);

  const handleTogglePin = useCallback(async (id: number) => {
    try {
      await invoke("toggle_pin", { id });
      fetchClipsRef.current();
    } catch (e) {
      console.error("收藏失败:", e);
    }
  }, []);

  const handleDelete = useCallback(async (id: number) => {
    try {
      await invoke("delete_clip", { id });
      fetchClipsRef.current();
    } catch (e) {
      console.error("删除失败:", e);
    }
  }, []);

  return {
    clips,
    selectedIdx,
    setSelectedIdx,
    clipsRef,
    selectedIdxRef,
    handleCopy,
    handleTogglePin,
    handleDelete,
  };
}
