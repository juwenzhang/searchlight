import type {
  SearchlightBm25Options,
  SearchlightRelatedSuggestion,
  SearchlightSearchOptions,
} from '../types';
import { isWorkerResponse, type WorkerRequest, type WorkerResponse } from './protocol';

let requestCounter = 0;

function nextRequestId() {
  requestCounter += 1;
  return `searchlight-${requestCounter}`;
}

export type WorkerSearchClientOptions = {
  worker?: Worker;
  workerUrl?: string | URL;
};

export class WorkerSearchClient {
  private readonly worker: Worker;
  private readonly pending = new Map<
    string,
    {
      resolve: (response: WorkerResponse) => void;
      reject: (error: Error) => void;
    }
  >();
  private readyPromise: Promise<void> | undefined;

  constructor(options: WorkerSearchClientOptions = {}) {
    this.worker =
      options.worker ??
      new Worker(options.workerUrl ?? defaultWorkerUrl(), {
        type: 'module',
        name: 'searchlight-worker',
      });

    this.worker.addEventListener('message', (event: MessageEvent<unknown>) => {
      if (!isWorkerResponse(event.data)) return;
      const pending = this.pending.get(event.data.id);
      if (!pending) return;
      this.pending.delete(event.data.id);
      if (event.data.type === 'error') {
        pending.reject(new Error(event.data.message));
        return;
      }
      pending.resolve(event.data);
    });

    this.worker.addEventListener('error', (event) => {
      const error = new Error(event.message || 'Searchlight worker failed');
      for (const [, pending] of this.pending) {
        pending.reject(error);
      }
      this.pending.clear();
    });
  }

  get rawWorker() {
    return this.worker;
  }

  async init(options: {
    documents: string[];
    wasmModule?: string;
    bm25?: SearchlightBm25Options;
    searchOptions?: SearchlightSearchOptions;
  }) {
    this.readyPromise = this.request({
      id: nextRequestId(),
      type: 'init',
      documents: options.documents,
      wasmModule: options.wasmModule,
      bm25: options.bm25,
      searchOptions: options.searchOptions,
    }).then(() => undefined);
    await this.readyPromise;
  }

  async search(query: string, options?: SearchlightSearchOptions) {
    await this.ensureReady();
    const response = await this.request({
      id: nextRequestId(),
      type: 'search',
      query,
      options,
    });
    if (response.type !== 'search') {
      throw new Error('Unexpected worker response for search');
    }
    return response.results;
  }

  async suggest(prefix: string, usePinyin = true) {
    await this.ensureReady();
    const response = await this.request({
      id: nextRequestId(),
      type: 'suggest',
      prefix,
      usePinyin,
    });
    if (response.type !== 'suggest') {
      throw new Error('Unexpected worker response for suggest');
    }
    return response.suggestions;
  }

  async suggestRelated(query: string, limit = 10): Promise<SearchlightRelatedSuggestion[]> {
    await this.ensureReady();
    const response = await this.request({
      id: nextRequestId(),
      type: 'suggestRelated',
      query,
      limit,
    });
    if (response.type !== 'suggestRelated') {
      throw new Error('Unexpected worker response for suggestRelated');
    }
    return response.suggestions;
  }

  async reindex(documents: string[]) {
    await this.ensureReady();
    await this.request({
      id: nextRequestId(),
      type: 'reindex',
      documents,
    });
  }

  async clear() {
    await this.ensureReady();
    await this.request({
      id: nextRequestId(),
      type: 'clear',
    });
  }

  dispose() {
    void this.request({
      id: nextRequestId(),
      type: 'dispose',
    }).catch(() => undefined);
    this.worker.terminate();
    this.readyPromise = undefined;
  }

  private async ensureReady() {
    if (!this.readyPromise) {
      throw new Error('Searchlight worker is not initialized. Call init() first.');
    }
    await this.readyPromise;
  }

  private request(message: WorkerRequest): Promise<WorkerResponse> {
    return new Promise((resolve, reject) => {
      this.pending.set(message.id, { resolve, reject });
      this.worker.postMessage(message);
    });
  }
}

export function defaultWorkerUrl() {
  return new URL('./searchlight.worker.js', import.meta.url);
}

export function createWorkerSearchClient(options?: WorkerSearchClientOptions) {
  return new WorkerSearchClient(options);
}
