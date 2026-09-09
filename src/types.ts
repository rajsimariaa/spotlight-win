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
  | "Timezone";

export interface SearchResponse {
  results: SearchResult[];
  query_time_ms: number;
  total_results: number;
}

export interface AppEntry {
  name: string;
  path: string;
  icon_path: string | null;
  app_type: "StartMenuShortcut" | "UwpApp" | "SystemBinary";
}
