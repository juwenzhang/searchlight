import type {
  SearchEngine,
  SearchlightDocumentMapper,
  SearchlightHit,
  SearchlightProviderOptions,
  SearchlightRelatedSuggestion,
  SearchlightResult,
  SearchlightSearchOptions,
} from './types';
import { createSearchlightEngine } from './core';

export const defaultSearchOptions: Required<SearchlightSearchOptions> = {
  fuzzy: true,
  maxEditDistance: 2,
  usePinyin: true,
  highlight: true,
  limit: 20,
  enableCache: true,
  explain: false,
};

export function defaultGetText(document: unknown) {
  return typeof document === 'string' ? document : JSON.stringify(document) ?? '';
}

export class LocalSearchProvider<TDocument = string> {
  private engine?: SearchEngine;
  private documents: TDocument[];
  private readonly getText: SearchlightDocumentMapper<TDocument>;
  private readonly options: SearchlightProviderOptions<TDocument>;

  constructor(options: SearchlightProviderOptions<TDocument> = {}) {
    this.options = options;
    this.documents = Array.from(options.documents ?? []);
    this.getText = options.getText ?? ((document) => defaultGetText(document));
  }

  get ready() {
    return Boolean(this.engine);
  }

  get docCount() {
    return this.engine?.docCount() ?? 0;
  }

  get rawEngine() {
    return this.engine;
  }

  async init() {
    this.engine = await createSearchlightEngine({ wasmModule: this.options.wasmModule, bm25: this.options.bm25 });
    this.reindex(this.documents);
  }

  reindex(documents: readonly TDocument[] = this.documents) {
    const engine = this.getEngine();
    this.documents = Array.from(documents);
    engine.clear();

    if (this.documents.length > 0) {
      engine.indexBatch(this.documents.map((document, index) => this.getText(document, index)));
    }
  }

  clear() {
    const engine = this.getEngine();
    this.documents = [];
    engine.clear();
  }

  search(query: string, options: SearchlightSearchOptions = {}) {
    const engine = this.getEngine();

    if (!query.trim()) return [];

    const results = engine.searchWithOptions(query, {
      ...defaultSearchOptions,
      ...this.options.searchOptions,
      ...options,
    }) as SearchlightResult[];

    return this.attachDocuments(results);
  }

  batchSearch(queries: readonly string[], options: SearchlightSearchOptions = {}) {
    const engine = this.getEngine();

    const results = engine.batchSearch(Array.from(queries), {
      ...defaultSearchOptions,
      ...this.options.searchOptions,
      ...options,
    }) as SearchlightResult[][];

    return results.map((items) => this.attachDocuments(items));
  }

  suggest(prefix: string, usePinyin = true) {
    const engine = this.getEngine();
    if (!prefix.trim()) return [];
    return (usePinyin ? engine.suggestWithPinyin(prefix) : engine.suggest(prefix)) as string[];
  }

  suggestRelated(query: string, limit = 10) {
    const engine = this.getEngine();
    if (!query.trim() || limit <= 0) return [];
    return engine.suggestRelated(query, limit) as SearchlightRelatedSuggestion[];
  }

  dispose() {
    this.engine?.free();
    this.engine = undefined;
  }

  private attachDocuments(results: SearchlightResult[]): Array<SearchlightHit<TDocument>> {
    return results.map((result) => ({
      ...result,
      item: this.documents[result.doc_id],
    }));
  }

  private getEngine() {
    if (!this.engine) {
      throw new Error('Searchlight engine is not initialized. Call init() before using it.');
    }
    return this.engine;
  }
}
