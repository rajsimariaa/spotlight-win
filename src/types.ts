export interface SearchResult {
  id: string;
  name: string;
  path: string;
  category: SearchResultCategory;
  icon: string | null;
  score: number;
  metadata: string | null;
}

export type SearchResultCategory =
  | "Application"
  | "File"
  | "Calculator"
  | "Action"
  | "Conversion"
  | "Timezone"
  | "WebSearch";

export interface SearchResponse {
  results: SearchResult[];
  query_time_ms: number;
  total_results: number;
}
