/// <reference lib="webworker" />

import init, { SearchEngine } from '@luhanxin/searchlight';
import { defaultSearchOptions } from '../provider';
import type {
  SearchlightRelatedSuggestion,
  SearchlightResult,
  SearchlightSearchOptions,
} from '../types';
import type { WorkerRequest, WorkerResponse } from './protocol';

declare const self: DedicatedWorkerGlobalScope;

let engine: SearchEngine | undefined;
let defaultOptions: SearchlightSearchOptions = { ...defaultSearchOptions };

async function ensureEngine(
  documents: string[],
  wasmModule?: string,
  bm25?: { k1: number; b: number },
) {
  if (wasmModule) {
    await init(wasmModule);
  } else {
    await init();
  }

  engine?.free();
  engine = bm25 ? SearchEngine.withBm25Params(bm25.k1, bm25.b) : new SearchEngine();

  if (documents.length > 0) {
    engine.indexBatch(documents);
  }
}

function post(response: WorkerResponse) {
  self.postMessage(response);
}

function searchOptions(options?: SearchlightSearchOptions) {
  return {
    ...defaultOptions,
    ...options,
  };
}

self.addEventListener('message', (event: MessageEvent<WorkerRequest>) => {
  const message = event.data;

  void (async () => {
    try {
      switch (message.type) {
        case 'init': {
          if (message.searchOptions) {
            defaultOptions = { ...defaultOptions, ...message.searchOptions };
          }
          await ensureEngine(message.documents, message.wasmModule, message.bm25);
          post({ id: message.id, type: 'ready' });
          break;
        }
        case 'search': {
          if (!engine) throw new Error('Searchlight worker is not initialized');
          const results = engine.searchWithOptions(
            message.query,
            searchOptions(message.options),
          ) as SearchlightResult[];
          post({ id: message.id, type: 'search', results });
          break;
        }
        case 'suggest': {
          if (!engine) throw new Error('Searchlight worker is not initialized');
          const suggestions = (
            message.usePinyin ?? true
              ? engine.suggestWithPinyin(message.prefix)
              : engine.suggest(message.prefix)
          ) as string[];
          post({ id: message.id, type: 'suggest', suggestions });
          break;
        }
        case 'suggestRelated': {
          if (!engine) throw new Error('Searchlight worker is not initialized');
          const suggestions = engine.suggestRelated(
            message.query,
            message.limit ?? 10,
          ) as SearchlightRelatedSuggestion[];
          post({ id: message.id, type: 'suggestRelated', suggestions });
          break;
        }
        case 'reindex': {
          if (!engine) throw new Error('Searchlight worker is not initialized');
          engine.clear();
          if (message.documents.length > 0) {
            engine.indexBatch(message.documents);
          }
          post({ id: message.id, type: 'ready' });
          break;
        }
        case 'clear': {
          engine?.clear();
          post({ id: message.id, type: 'ready' });
          break;
        }
        case 'dispose': {
          engine?.free();
          engine = undefined;
          post({ id: message.id, type: 'ready' });
          break;
        }
        default:
          throw new Error(`Unknown worker message type: ${(message as WorkerRequest).type}`);
      }
    } catch (error: unknown) {
      post({
        id: message.id,
        type: 'error',
        message: error instanceof Error ? error.message : String(error),
      });
    }
  })();
});
