import { useState, useEffect, useRef, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import SearchInput from "./components/SearchInput";
import ResultsList from "./components/ResultsList";
import PreviewPanel from "./components/PreviewPanel";
import { SearchResult, SearchResponse } from "./types";

export default function App() {
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<SearchResult[]>([]);
  const [selectedIndex, setSelectedIndex] = useState(0);
  const [queryTime, setQueryTime] = useState(0);
  const [selectedResult, setSelectedResult] = useState<SearchResult | null>(null);
  const debounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const inputRef = useRef<HTMLInputElement | null>(null);

  useEffect(() => {
    const unlistenShow = listen("spotlight-show", () => {
      resetAndFocus();
    });
    const unlistenHide = listen("spotlight-hide", () => {
      setQuery("");
      setResults([]);
    });
    return () => {
      unlistenShow.then((fn) => fn());
      unlistenHide.then((fn) => fn());
    };
  }, []);

  const resetAndFocus = useCallback(() => {
    setQuery("");
    setResults([]);
    setSelectedIndex(0);
    setQueryTime(0);
    setSelectedResult(null);
    // Resize to minimal
    invoke("resize_window", { height: 60 }).catch(() => {});
    setTimeout(() => inputRef.current?.focus(), 30);
  }, []);

  // Resize window dynamically
  useEffect(() => {
    const count = results.length;
    if (count === 0) {
      const h = query.trim() !== "" ? 100 : 60;
      invoke("resize_window", { height: h }).catch(() => {});
    } else {
      const h = Math.min(60 + count * 40 + 30, 420);
      invoke("resize_window", { height: h }).catch(() => {});
    }
  }, [results, query]);

  const handleSearch = useCallback((value: string) => {
    setQuery(value);
    setSelectedIndex(0);
    if (debounceRef.current) clearTimeout(debounceRef.current);
    debounceRef.current = setTimeout(async () => {
      if (value.trim() === "") {
        setResults([]);
        setQueryTime(0);
        return;
      }
      try {
        const response: SearchResponse = await invoke("search_query", { query: value });
        setResults(response.results);
        setQueryTime(response.query_time_ms);
      } catch (err) {
        console.error("Search error:", err);
        setResults([]);
      }
    }, 5);
  }, []);

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      switch (e.key) {
        case "ArrowDown":
          e.preventDefault();
          setSelectedIndex((prev) => Math.min(prev + 1, results.length - 1));
          break;
        case "ArrowUp":
          e.preventDefault();
          setSelectedIndex((prev) => Math.max(prev - 1, 0));
          break;
        case "Enter":
          e.preventDefault();
          if (results[selectedIndex]) handleSelectResult(results[selectedIndex]);
          break;
        case "Escape":
          e.preventDefault();
          invoke("toggle_window").catch(() => {});
          break;
      }
    },
    [results, selectedIndex]
  );

  const handleSelectResult = async (result: SearchResult) => {
    try {
      if (result.category === "Action") {
        await invoke("execute_action", { actionId: result.metadata || "" });
      } else {
        await invoke("execute_action", { actionId: `open:${result.path}` });
      }
      invoke("toggle_window").catch(() => {});
    } catch (err) {
      console.error("Execute error:", err);
    }
  };

  const groupedResults = results.reduce<Record<string, SearchResult[]>>((acc, result) => {
    if (!acc[result.category]) acc[result.category] = [];
    acc[result.category].push(result);
    return acc;
  }, {});

  useEffect(() => {
    if (results[selectedIndex]) setSelectedResult(results[selectedIndex]);
  }, [selectedIndex, results]);

  return (
    <div className="spotlight-container flex flex-col">
      <div className="border-b border-white/10">
        <SearchInput
          ref={inputRef}
          value={query}
          onChange={handleSearch}
          onKeyDown={handleKeyDown}
          queryTime={queryTime}
          resultCount={results.length}
        />
      </div>

      {results.length > 0 && (
        <div className="flex overflow-hidden" style={{ maxHeight: 360 }}>
          <div className="flex-1 overflow-hidden">
            <ResultsList
              groupedResults={groupedResults}
              selectedIndex={selectedIndex}
              onSelect={setSelectedIndex}
              onExecute={handleSelectResult}
              query={query}
            />
          </div>
          {selectedResult && (
            <PreviewPanel result={selectedResult} query={query} />
          )}
        </div>
      )}

      {query.trim() !== "" && results.length === 0 && (
        <div className="py-4 text-center text-text-muted text-sm">
          No results found
        </div>
      )}
    </div>
  );
}
