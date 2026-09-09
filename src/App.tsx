import { useState, useEffect, useRef, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { SearchResult, SearchResponse } from "./types";

const SEARCH_H = 54;
const ROW_H = 42;
const LABEL_H = 26;
const MAX_H = 440;

const catIcon: Record<string, string> = {
  Application: "📱", Action: "⚡", Calculator: "🧮",
  File: "📄", WebSearch: "🌐", Conversion: "🔄", Timezone: "🕐",
};

const catDot: Record<string, string> = {
  Application: "app", Action: "action", Calculator: "calc",
  File: "file", WebSearch: "web", Conversion: "conv", Timezone: "tz",
};

const fileIcons: Record<string, string> = {
  pdf: "📕", doc: "📘", docx: "📘", xls: "📗", xlsx: "📗", ppt: "📙", pptx: "📙",
  txt: "📝", csv: "📊", md: "📝",
  jpg: "🖼️", jpeg: "🖼️", png: "🖼️", gif: "🖼️", svg: "🖼️", webp: "🖼️",
  mp3: "🎵", wav: "🎵", flac: "🎵", aac: "🎵", m4a: "🎵",
  mp4: "🎬", mkv: "🎬", avi: "🎬", mov: "🎬", wmv: "🎬", webm: "🎬",
  js: "💛", ts: "💙", tsx: "💙", jsx: "💛", py: "🐍", rs: "🦀", go: "🔷",
  java: "☕", c: "⚙️", cpp: "⚙️", cs: "🟣", html: "🌐", css: "🎨",
  json: "📋", xml: "📋", yaml: "📋", yml: "📋", toml: "📋",
  zip: "📦", rar: "📦", "7z": "📦",
};

function getFileIcon(metadata: string | null): string {
  if (!metadata) return "📄";
  const ext = metadata.replace("file:", "");
  return fileIcons[ext] || "📄";
}

function getShortPath(fullPath: string): string {
  const parts = fullPath.replace(/\\/g, "/").split("/");
  if (parts.length <= 2) return fullPath;
  return parts.slice(-2).join("/");
}

export default function App() {
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<SearchResult[]>([]);
  const [sel, setSel] = useState(0);
  const debounce = useRef<ReturnType<typeof setTimeout> | null>(null);
  const inputRef = useRef<HTMLInputElement>(null);
  const resultsRef = useRef<HTMLDivElement>(null);
  const selRef = useRef(0);

  useEffect(() => {
    const u1 = listen("spotlight-show", () => {
      setQuery(""); setResults([]); setSel(0); selRef.current = 0;
      setTimeout(() => inputRef.current?.focus(), 30);
    });
    const u2 = listen("spotlight-hide", () => {
      setQuery(""); setResults([]); selRef.current = 0;
    });
    return () => { u1.then(f => f()); u2.then(f => f()); };
  }, []);

  const scrollToSel = useCallback((idx: number) => {
    const container = resultsRef.current;
    if (!container) return;
    const row = container.querySelector(`[data-idx="${idx}"]`);
    if (row) row.scrollIntoView({ block: "nearest", behavior: "smooth" });
  }, []);

  const resize = useCallback((count: number, hasWeb: boolean) => {
    if (count === 0 && !hasWeb) {
      invoke("resize_window", { height: SEARCH_H });
    } else {
      const cats = new Set(results.map(r => r.category)).size;
      const webH = hasWeb ? 38 : 0;
      const h = SEARCH_H + (count * ROW_H) + (cats * LABEL_H) + webH + 12;
      invoke("resize_window", { height: Math.min(h, MAX_H) });
    }
  }, [results]);

  useEffect(() => resize(results.length, results.length === 0 && query.trim().length > 0), [results, query, resize]);
  useEffect(() => { scrollToSel(sel); }, [sel, scrollToSel]);

  const search = useCallback((q: string) => {
    setQuery(q); setSel(0); selRef.current = 0;
    if (debounce.current) clearTimeout(debounce.current);
    debounce.current = setTimeout(async () => {
      if (!q.trim()) { setResults([]); return; }
      try {
        const r: SearchResponse = await invoke("search_query", { query: q });
        setResults(r.results);
      } catch { setResults([]); }
    }, 3);
  }, []);

  const openWebSearch = useCallback((q: string) => {
    if (!q.trim()) return;
    const url = `https://www.google.com/search?q=${encodeURIComponent(q)}`;
    invoke("execute_action", { actionId: `web:${q}` }).catch(() => {});
    invoke("toggle_window").catch(() => {});
  }, []);

  const executeItem = useCallback((item: SearchResult) => {
    const id = item.category === "Action" ? (item.metadata || "") :
               `open:${item.path}`;
    invoke("execute_action", { actionId: id }).catch(() => {});
    invoke("toggle_window").catch(() => {});
  }, []);

  const onKey = useCallback((e: React.KeyboardEvent) => {
    const len = results.length;
    const noResults = len === 0;

    if (e.key === "ArrowDown") {
      e.preventDefault();
      if (len > 0) {
        const next = (selRef.current + 1) % len;
        selRef.current = next; setSel(next);
      }
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      if (len > 0) {
        const prev = selRef.current <= 0 ? len - 1 : selRef.current - 1;
        selRef.current = prev; setSel(prev);
      }
    } else if (e.key === "Enter") {
      e.preventDefault();
      if (noResults) {
        // No local results — open web search
        openWebSearch(query);
      } else if (results[selRef.current]) {
        executeItem(results[selRef.current]);
      }
    } else if (e.key === "Escape") {
      e.preventDefault();
      invoke("toggle_window").catch(() => {});
    }
  }, [results, query, executeItem, openWebSearch]);

  const groups: Record<string, SearchResult[]> = {};
  results.forEach(r => { (groups[r.category] ||= []).push(r); });
  const order = ["Application", "Action", "Calculator", "Conversion", "Timezone", "File"];
  const showWeb = results.length === 0 && query.trim().length > 0;

  return (
    <div className="spotlight">
      <div className="search-bar">
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.2" strokeLinecap="round">
          <circle cx="10.5" cy="10.5" r="7.5"/><line x1="21" y1="21" x2="15.8" y2="15.8"/>
        </svg>
        <input
          ref={inputRef}
          value={query}
          onChange={e => search(e.target.value)}
          onKeyDown={onKey}
          placeholder="Spotlight Search"
          autoFocus
          autoComplete="off"
          autoCorrect="off"
          spellCheck={false}
        />
      </div>

      {results.length > 0 && (
        <>
          <div className="divider" />
          <div className="results" ref={resultsRef}>
            {order.map(cat => {
              const items = groups[cat];
              if (!items) return null;
              return (
                <div key={cat}>
                  <div className="category-header">
                    <div className={`category-dot ${catDot[cat] || "app"}`} />
                    <span className="category-label">{cat}</span>
                  </div>
                  {items.map(item => {
                    const idx = results.indexOf(item);
                    const isActive = idx === sel;
                    const icon = cat === "File" ? getFileIcon(item.metadata) : (catIcon[cat] || "📁");
                    const shortPath = cat === "File"
                      ? getShortPath(item.path)
                      : (item.path && item.path !== item.name
                          ? item.path.replace(/\\/g, "/").split("/").slice(-2).join("/")
                          : "");
                    return (
                      <div
                        key={`${item.id}-${idx}`}
                        data-idx={idx}
                        className={`result-row${isActive ? " active" : ""}`}
                        onMouseEnter={() => { selRef.current = idx; setSel(idx); }}
                        onClick={() => executeItem(item)}
                      >
                        <div className="result-icon">{icon}</div>
                        <div className="result-info">
                          <span className="result-name">{item.name}</span>
                          {shortPath && <span className="result-meta">{shortPath}</span>}
                        </div>
                        <div className="result-right">
                          {isActive && <span className="result-shortcut">↵</span>}
                        </div>
                      </div>
                    );
                  })}
                </div>
              );
            })}
          </div>
        </>
      )}

      {showWeb && (
        <>
          <div className="divider" />
          <div className="web-search-footer" onClick={() => openWebSearch(query)}>
            <div className="result-icon">🌐</div>
            <div className="result-info">
              <span className="result-name">Search &quot;{query}&quot; on the web</span>
              <span className="result-meta">Press Enter to open in browser</span>
            </div>
            <div className="result-right">
              <span className="result-shortcut">↵</span>
            </div>
          </div>
        </>
      )}

      {query.trim() !== "" && results.length === 0 && (
        <div className="divider" />
      )}
      {query.trim() !== "" && results.length === 0 && (
        <div className="empty-state">No results for &quot;{query}&quot; — press Enter to search the web</div>
      )}
    </div>
  );
}
