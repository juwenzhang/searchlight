import type { InitInput, SearchEngine } from '@luhanxin/searchlight';

export type SearchlightSearchOptions = {
  fuzzy?: boolean;
  maxEditDistance?: number;
  usePinyin?: boolean;
  highlight?: boolean;
  limit?: number;
};

export type SearchlightResult = {
  doc_id: number;
  score: number;
  document: string;
  snippet?: string | null;
  match_positions: Array<[number, number]>;
  matched_terms: string[];
};

export type SearchlightHit<TDocument = string> = SearchlightResult & {
  item?: TDocument;
};

export type SearchlightDocumentMapper<TDocument> = (document: TDocument, index: number) => string;

export type SearchlightBm25Options = {
  k1: number;
  b: number;
};

export type SearchlightProviderOptions<TDocument = string> = {
  documents?: readonly TDocument[];
  getText?: SearchlightDocumentMapper<TDocument>;
  wasmModule?: InitInput | Promise<InitInput>;
  bm25?: SearchlightBm25Options;
  searchOptions?: SearchlightSearchOptions;
};

export type SearchlightStatus = 'idle' | 'loading' | 'ready' | 'error';

export type UseSearchlightOptions<TDocument = string> = SearchlightProviderOptions<TDocument> & {
  initialQuery?: string;
  autoSearch?: boolean;
  suggest?: boolean;
};

export type UseSearchlightState<TDocument = string> = {
  status: SearchlightStatus;
  ready: boolean;
  loading: boolean;
  error?: Error;
  query: string;
  results: Array<SearchlightHit<TDocument>>;
  suggestions: string[];
  setQuery(query: string): void;
  search(query?: string, options?: SearchlightSearchOptions): Array<SearchlightHit<TDocument>>;
  suggest(prefix?: string): string[];
  reindex(documents: readonly TDocument[]): void;
  clear(): void;
};

export type UseSearchlightEngineState = {
  status: SearchlightStatus;
  ready: boolean;
  loading: boolean;
  error?: Error;
  engine?: SearchEngine;
};

export type { InitInput, SearchEngine };
