import { createWorkerSearchClient, type WorkerSearchClient, type WorkerSearchClientOptions } from './worker/client';
import type {
  SearchlightDocumentMapper,
  SearchlightHit,
  SearchlightProviderOptions,
  SearchlightRelatedSuggestion,
  SearchlightResult,
  SearchlightSearchOptions,
} from './types';
import { defaultGetText } from './provider';

export type WorkerSearchProviderOptions<TDocument = string> = SearchlightProviderOptions<TDocument> &
  WorkerSearchClientOptions & {
    wasmModuleUrl?: string;
  };

export class WorkerSearchProvider<TDocument = string> {
  private client: WorkerSearchClient;
  private documents: TDocument[];
  private readonly getText: SearchlightDocumentMapper<TDocument>;
  private readonly options: WorkerSearchProviderOptions<TDocument>;
  private initialized = false;

  constructor(options: WorkerSearchProviderOptions<TDocument> = {}) {
    this.options = options;
    this.documents = Array.from(options.documents ?? []);
    this.getText = options.getText ?? ((document) => defaultGetText(document));
    this.client = options.worker
      ? createWorkerSearchClient({ worker: options.worker })
      : createWorkerSearchClient({ workerUrl: options.workerUrl });
  }

  get ready() {
    return this.initialized;
  }

  get rawClient() {
    return this.client;
  }

  async init() {
    await this.client.init({
      documents: this.documents.map((document, index) => this.getText(document, index)),
      wasmModule: this.options.wasmModuleUrl,
      bm25: this.options.bm25,
      searchOptions: this.options.searchOptions,
    });
    this.initialized = true;
  }

  async reindex(documents: readonly TDocument[] = this.documents) {
    this.documents = Array.from(documents);
    const texts = this.documents.map((document, index) => this.getText(document, index));
    await this.client.reindex(texts);
  }

  async clear() {
    await this.client.clear();
    this.documents = [];
  }

  async search(query: string, options: SearchlightSearchOptions = {}) {
    if (!query.trim()) return [];

    const results = await this.client.search(query, {
      ...this.options.searchOptions,
      ...options,
    });

    return this.attachDocuments(results);
  }

  async batchSearch(queries: readonly string[], options: SearchlightSearchOptions = {}) {
    const batches = await Promise.all(
      queries.map((query) => this.search(query, options)),
    );
    return batches;
  }

  async suggest(prefix: string, usePinyin = true) {
    if (!prefix.trim()) return [];
    return this.client.suggest(prefix, usePinyin);
  }

  async suggestRelated(query: string, limit = 10): Promise<SearchlightRelatedSuggestion[]> {
    if (!query.trim() || limit <= 0) return [];
    return this.client.suggestRelated(query, limit);
  }

  dispose() {
    this.client.dispose();
    this.initialized = false;
  }

  private attachDocuments(results: SearchlightResult[]): Array<SearchlightHit<TDocument>> {
    return results.map((result) => ({
      ...result,
      item: this.documents[result.doc_id],
    }));
  }
}
