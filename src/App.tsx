import { useState, useEffect, useRef, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { SearchResult, SearchResponse } from "./types";

const SEARCH_H = 52;
const ROW_H = 38;
const LABEL_H = 22;
const MAX_H = 400;

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
      setTimeout(() => inputRef.current?.focus(), 20);
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
    if (row) row.scrollIntoView({ block: "nearest" });
  }, []);

  const resize = useCallback((count: number) => {
    if (count === 0) {
      invoke("resize_window", { height: SEARCH_H });
    } else {
      const cats = new Set(results.map(r => r.category)).size;
      const h = SEARCH_H + (count * ROW_H) + (cats * LABEL_H) + 8;
      invoke("resize_window", { height: Math.min(h, MAX_H) });
    }
  }, [results]);

  useEffect(() => resize(results.length), [results, resize]);

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

  const onKey = useCallback((e: React.KeyboardEvent) => {
    const len = results.length;
    if (e.key === "ArrowDown") {
      e.preventDefault();
      const next = selRef.current + 1;
      if (next < len) { selRef.current = next; setSel(next); }
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      const prev = selRef.current - 1;
      if (prev >= 0) { selRef.current = prev; setSel(prev); }
    } else if (e.key === "Enter") {
      e.preventDefault();
      if (results[selRef.current]) {
        const r = results[selRef.current];
        const id = r.category === "Action" ? (r.metadata || "") : `open:${r.path}`;
        invoke("execute_action", { actionId: id }).catch(() => {});
        invoke("toggle_window").catch(() => {});
      }
    } else if (e.key === "Escape") {
      e.preventDefault();
      invoke("toggle_window").catch(() => {});
    }
  }, [results]);

  const groups: Record<string, SearchResult[]> = {};
  results.forEach(r => { (groups[r.category] ||= []).push(r); });
  const order = ["Application", "Action", "Calculator", "Conversion", "Timezone", "File", "WebSearch"];

  return (
    <div className="spotlight">
      <div className="search-bar">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="rgba(255,255,255,0.35)" strokeWidth="2" strokeLinecap="round">
          <circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/>
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
                  <div className="category-label">{cat}</div>
                  {items.map(item => {
                    const idx = results.indexOf(item);
                    const isActive = idx === sel;
                    return (
                      <div
                        key={item.id + idx}
                        data-idx={idx}
                        className={`result-row${isActive ? " active" : ""}`}
                        onMouseEnter={() => { selRef.current = idx; setSel(idx); }}
                        onClick={() => {
                          const id = item.category === "Action" ? (item.metadata || "") : `open:${item.path}`;
                          invoke("execute_action", { actionId: id }).catch(() => {});
                          invoke("toggle_window").catch(() => {});
                        }}
                      >
                        <div className="result-icon" style={{ background: isActive ? "rgba(255,255,255,0.15)" : "rgba(255,255,255,0.04)" }}>
                          {cat === "Application" ? "📱" : cat === "Action" ? "⚡" : cat === "Calculator" ? "🧮" : cat === "File" ? "📄" : cat === "Conversion" ? "🔄" : cat === "WebSearch" ? "🌐" : "🕐"}
                        </div>
                        <span className="result-name">{item.name}</span>
                        {item.path && item.path !== item.name && (
                          <span className="result-path">{item.path}</span>
                        )}
                        {isActive && <span className="result-shortcut">↵</span>}
                      </div>
                    );
                  })}
                </div>
              );
            })}
          </div>
        </>
      )}

      {query.trim() !== "" && results.length === 0 && (
        <div className="divider" />
      )}
      {query.trim() !== "" && results.length === 0 && (
        <div className="empty-state">No results for &quot;{query}&quot;</div>
      )}
    </div>
  );
}
