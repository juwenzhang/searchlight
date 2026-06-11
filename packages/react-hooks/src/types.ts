import type { InitInput, SearchEngine } from '@luhanxin/searchlight';

export type SearchlightSearchOptions = {
  fuzzy?: boolean;
  maxEditDistance?: number;
  usePinyin?: boolean;
  highlight?: boolean;
  limit?: number;
  enableCache?: boolean;
  explain?: boolean;
};

export type ScoreBreakdown = {
  bm25: number;
  proximity: number;
  coverage: number;
  phrase: number;
  pinyin: number;
  total: number;
};

export type MatchReason = {
  code: string;
  message: string;
  terms: string[];
};

export type SearchlightResult = {
  doc_id: number;
  score: number;
  document: string;
  snippet?: string | null;
  match_positions: Array<[number, number]>;
  matched_terms: string[];
  score_breakdown?: ScoreBreakdown | null;
  match_reasons?: MatchReason[] | null;
};

export type SearchlightRelatedSuggestion = {
  term: string;
  score: number;
  doc_frequency: number;
  total_frequency: number;
  source_doc_ids: number[];
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
  related?: boolean;
  relatedLimit?: number;
};

export type UseSearchlightState<TDocument = string> = {
  status: SearchlightStatus;
  ready: boolean;
  loading: boolean;
  error?: Error;
  query: string;
  results: Array<SearchlightHit<TDocument>>;
  suggestions: string[];
  relatedSuggestions: SearchlightRelatedSuggestion[];
  setQuery(query: string): void;
  search(query?: string, options?: SearchlightSearchOptions): Array<SearchlightHit<TDocument>>;
  suggest(prefix?: string): string[];
  suggestRelated(query?: string, limit?: number): SearchlightRelatedSuggestion[];
  reindex(documents: readonly TDocument[]): void;
  clear(): void;
};

export type UseSearchlightWorkerState<TDocument = string> = Omit<
  UseSearchlightState<TDocument>,
  'search' | 'suggest' | 'suggestRelated' | 'reindex' | 'clear'
> & {
  search(
    query?: string,
    options?: SearchlightSearchOptions,
  ): Promise<Array<SearchlightHit<TDocument>>>;
  suggest(prefix?: string): Promise<string[]>;
  suggestRelated(query?: string, limit?: number): Promise<SearchlightRelatedSuggestion[]>;
  reindex(documents: readonly TDocument[]): Promise<void>;
  clear(): Promise<void>;
};

export type UseSearchlightEngineState = {
  status: SearchlightStatus;
  ready: boolean;
  loading: boolean;
  error?: Error;
  engine?: SearchEngine;
};

export type { InitInput, SearchEngine };
