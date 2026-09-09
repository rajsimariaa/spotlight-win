import { useRef, useEffect } from "react";
import {
  AppWindow,
  File,
  Calculator,
  Zap,
  Ruler,
  Clock,
} from "lucide-react";
import { SearchResult, SearchResultCategory } from "../types";

interface ResultsListProps {
  groupedResults: Record<string, SearchResult[]>;
  selectedIndex: number;
  onSelect: (index: number) => void;
  onExecute: (result: SearchResult) => void;
  query: string;
}

const categoryIcons: Record<SearchResultCategory, React.ReactNode> = {
  Application: <AppWindow className="w-4 h-4" />,
  File: <File className="w-4 h-4" />,
  Calculator: <Calculator className="w-4 h-4" />,
  Action: <Zap className="w-4 h-4" />,
  Conversion: <Ruler className="w-4 h-4" />,
  Timezone: <Clock className="w-4 h-4" />,
};

const categoryOrder: SearchResultCategory[] = [
  "Calculator",
  "Conversion",
  "Timezone",
  "Application",
  "File",
  "Action",
];

export default function ResultsList({
  groupedResults,
  selectedIndex,
  onSelect,
  onExecute,
  query,
}: ResultsListProps) {
  const listRef = useRef<HTMLDivElement>(null);

  // Flatten results in category order for index tracking
  const flatResults: SearchResult[] = [];
  for (const category of categoryOrder) {
    if (groupedResults[category]) {
      flatResults.push(...groupedResults[category]);
    }
  }

  // Scroll selected item into view
  useEffect(() => {
    const selectedElement = listRef.current?.querySelector(`[data-index="${selectedIndex}"]`);
    if (selectedElement) {
      selectedElement.scrollIntoView({ block: "nearest", behavior: "smooth" });
    }
  }, [selectedIndex]);

  if (!query || flatResults.length === 0) {
    return (
      <div className="flex items-center justify-center h-full text-text-muted text-sm">
        {query ? "No results found" : "Type to search"}
      </div>
    );
  }

  let globalIndex = 0;

  return (
    <div ref={listRef} className="spotlight-results">
      {categoryOrder.map((category) => {
        const items = groupedResults[category];
        if (!items || items.length === 0) return null;

        return (
          <div key={category}>
            <div className="category-header flex items-center gap-2">
              {categoryIcons[category]}
              <span>{category}</span>
            </div>
            {items.map((result) => {
              const currentIndex = globalIndex++;
              return (
                <ResultItem
                  key={result.id}
                  result={result}
                  index={currentIndex}
                  isSelected={currentIndex === selectedIndex}
                  onSelect={() => onSelect(currentIndex)}
                  onExecute={() => onExecute(result)}
                />
              );
            })}
          </div>
        );
      })}
    </div>
  );
}

interface ResultItemProps {
  result: SearchResult;
  index: number;
  isSelected: boolean;
  onSelect: () => void;
  onExecute: () => void;
}

function ResultItem({ result, index, isSelected, onSelect, onExecute }: ResultItemProps) {
  return (
    <div
      data-index={index}
      className={`result-item ${isSelected ? "active" : ""}`}
      onMouseEnter={onSelect}
      onClick={onExecute}
    >
      <div className="result-icon">
        {categoryIcons[result.category] || <File className="w-4 h-4" />}
      </div>
      <div className="flex-1 min-w-0 ml-3">
        <div className="text-sm font-medium truncate">{result.name}</div>
        <div className="text-xs text-text-secondary truncate">{result.path}</div>
      </div>
      {isSelected && (
        <div className="text-xs text-white/60 ml-2 flex-shrink-0">
          {result.category === "Action" ? "↵" : "↵"}
        </div>
      )}
    </div>
  );
}
