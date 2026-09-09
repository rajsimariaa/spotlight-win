import { forwardRef } from "react";
import { Search } from "lucide-react";

interface SearchInputProps {
  value: string;
  onChange: (value: string) => void;
  onKeyDown: (e: React.KeyboardEvent) => void;
  queryTime: number;
  resultCount: number;
}

const SearchInput = forwardRef<HTMLInputElement, SearchInputProps>(
  ({ value, onChange, onKeyDown }, ref) => {
    return (
      <div className="flex items-center gap-3 px-4 py-0" style={{ height: 50 }}>
        <Search className="w-4 h-4 text-text-secondary flex-shrink-0" />
        <input
          ref={ref}
          type="text"
          className="spotlight-input flex-1"
          placeholder="Spotlight Search"
          value={value}
          onChange={(e) => onChange(e.target.value)}
          onKeyDown={onKeyDown}
          autoFocus
          autoComplete="off"
          autoCorrect="off"
          autoCapitalize="off"
          spellCheck={false}
        />
      </div>
    );
  }
);

SearchInput.displayName = "SearchInput";

export default SearchInput;
