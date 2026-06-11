import type {
  SearchlightBm25Options,
  SearchlightRelatedSuggestion,
  SearchlightResult,
  SearchlightSearchOptions,
} from '../types';

export type WorkerInitMessage = {
  id: string;
  type: 'init';
  documents: string[];
  wasmModule?: string;
  bm25?: SearchlightBm25Options;
  searchOptions?: SearchlightSearchOptions;
};

export type WorkerSearchMessage = {
  id: string;
  type: 'search';
  query: string;
  options?: SearchlightSearchOptions;
};

export type WorkerSuggestMessage = {
  id: string;
  type: 'suggest';
  prefix: string;
  usePinyin?: boolean;
};

export type WorkerSuggestRelatedMessage = {
  id: string;
  type: 'suggestRelated';
  query: string;
  limit?: number;
};

export type WorkerReindexMessage = {
  id: string;
  type: 'reindex';
  documents: string[];
};

export type WorkerClearMessage = {
  id: string;
  type: 'clear';
};

export type WorkerDisposeMessage = {
  id: string;
  type: 'dispose';
};

export type WorkerRequest =
  | WorkerInitMessage
  | WorkerSearchMessage
  | WorkerSuggestMessage
  | WorkerSuggestRelatedMessage
  | WorkerReindexMessage
  | WorkerClearMessage
  | WorkerDisposeMessage;

export type WorkerReadyResponse = {
  id: string;
  type: 'ready';
};

export type WorkerSearchResponse = {
  id: string;
  type: 'search';
  results: SearchlightResult[];
};

export type WorkerSuggestResponse = {
  id: string;
  type: 'suggest';
  suggestions: string[];
};

export type WorkerSuggestRelatedResponse = {
  id: string;
  type: 'suggestRelated';
  suggestions: SearchlightRelatedSuggestion[];
};

export type WorkerErrorResponse = {
  id: string;
  type: 'error';
  message: string;
};

export type WorkerResponse =
  | WorkerReadyResponse
  | WorkerSearchResponse
  | WorkerSuggestResponse
  | WorkerSuggestRelatedResponse
  | WorkerErrorResponse;

export function isWorkerResponse(value: unknown): value is WorkerResponse {
  return (
    typeof value === 'object' &&
    value !== null &&
    'id' in value &&
    'type' in value &&
    typeof (value as WorkerResponse).id === 'string'
  );
}
