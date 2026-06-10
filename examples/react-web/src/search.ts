import init, { SearchEngine } from '@luhanxin/searchlight';
import { documents, toSearchText, type DemoDocument } from './data';

export type SearchOptions = {
  fuzzy?: boolean;
  maxEditDistance?: number;
  usePinyin?: boolean;
  highlight?: boolean;
  limit?: number;
};

export type SearchResult = {
  doc_id: number;
  score: number;
  document: string;
  snippet?: string | null;
  match_positions: Array<[number, number]>;
  matched_terms: string[];
};

export type SearchHit = SearchResult & {
  item?: DemoDocument;
};

export interface SearchProvider {
  init?(): Promise<void>;
  search(query: string, options?: SearchOptions): Promise<SearchHit[]>;
  suggest?(prefix: string): Promise<string[]>;
}

let wasmReady: Promise<void> | undefined;

function initSearchlightWasm() {
  wasmReady ??= init().then(() => undefined);
  return wasmReady;
}

function attachDocuments(results: SearchResult[]): SearchHit[] {
  return results.map((result) => ({
    ...result,
    item: documents[result.doc_id],
  }));
}

export class LocalSearchProvider implements SearchProvider {
  private engine?: SearchEngine;

  async init() {
    await initSearchlightWasm();
    this.engine = new SearchEngine();
    this.engine.indexBatch(documents.map(toSearchText));
  }

  async search(query: string, options: SearchOptions = {}) {
    if (!this.engine || !query.trim()) return [];

    const results = this.engine.searchWithOptions(query, {
      fuzzy: true,
      maxEditDistance: 2,
      usePinyin: true,
      highlight: true,
      limit: 20,
      ...options,
    }) as SearchResult[];

    return attachDocuments(results);
  }

  async suggest(prefix: string) {
    if (!this.engine || !prefix.trim()) return [];
    return this.engine.suggestWithPinyin(prefix) as string[];
  }
}

export class RemoteSearchProvider implements SearchProvider {
  private readonly endpoint: string;

  constructor(endpoint = '/api/search') {
    this.endpoint = endpoint;
  }

  async search(query: string, options: SearchOptions = {}) {
    if (!query.trim()) return [];

    const response = await fetch(this.endpoint, {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ query, options }),
    });

    if (!response.ok) {
      throw new Error(`远程搜索失败：${response.status}`);
    }

    return attachDocuments((await response.json()) as SearchResult[]);
  }
}

export function createSearchProvider(mode: 'local' | 'remote'): SearchProvider {
  return mode === 'local' ? new LocalSearchProvider() : new RemoteSearchProvider();
}
