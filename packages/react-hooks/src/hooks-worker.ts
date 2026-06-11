import { useCallback, useEffect, useRef, useState } from 'react';
import { WorkerSearchProvider, type WorkerSearchProviderOptions } from './worker-provider';
import type {
  SearchlightHit,
  SearchlightRelatedSuggestion,
  SearchlightSearchOptions,
  SearchlightStatus,
  UseSearchlightOptions,
  UseSearchlightWorkerState,
} from './types';

function toError(error: unknown) {
  return error instanceof Error ? error : new Error(String(error));
}

export type UseSearchlightWorkerOptions<TDocument = string> = UseSearchlightOptions<TDocument> &
  Pick<WorkerSearchProviderOptions<TDocument>, 'worker' | 'workerUrl' | 'wasmModuleUrl'>;

export function useSearchlightWorker<TDocument = string>(
  documents: readonly TDocument[],
  options: UseSearchlightWorkerOptions<TDocument> = {},
): UseSearchlightWorkerState<TDocument> {
  const providerRef = useRef<WorkerSearchProvider<TDocument> | undefined>(undefined);
  const [status, setStatus] = useState<SearchlightStatus>('idle');
  const [error, setError] = useState<Error>();
  const [query, setQuery] = useState(options.initialQuery ?? '');
  const [results, setResults] = useState<Array<SearchlightHit<TDocument>>>([]);
  const [suggestions, setSuggestions] = useState<string[]>([]);
  const [relatedSuggestions, setRelatedSuggestions] = useState<SearchlightRelatedSuggestion[]>([]);
  const autoSearch = options.autoSearch ?? true;
  const enableSuggest = options.suggest ?? true;
  const enableRelated = options.related ?? true;
  const relatedLimit = options.relatedLimit ?? 8;

  const search = useCallback(
    (nextQuery = query, searchOptions?: SearchlightSearchOptions) => {
      const provider = providerRef.current;
      if (!provider?.ready) return Promise.resolve([]);

      return provider
        .search(nextQuery, searchOptions)
        .then((nextResults) => {
          setResults(nextResults);
          setError(undefined);
          return nextResults;
        })
        .catch((err: unknown) => {
          const nextError = toError(err);
          setError(nextError);
          setStatus('error');
          return [];
        });
    },
    [query],
  );

  const suggest = useCallback(
    (prefix = query) => {
      const provider = providerRef.current;
      if (!provider?.ready) return Promise.resolve([]);

      return provider
        .suggest(prefix)
        .then((nextSuggestions) => {
          setSuggestions(nextSuggestions);
          return nextSuggestions;
        })
        .catch(() => {
          setSuggestions([]);
          return [];
        });
    },
    [query],
  );

  const suggestRelated = useCallback(
    (nextQuery = query, limit = relatedLimit) => {
      const provider = providerRef.current;
      if (!provider?.ready) return Promise.resolve([]);

      return provider
        .suggestRelated(nextQuery, limit)
        .then((nextRelatedSuggestions) => {
          setRelatedSuggestions(nextRelatedSuggestions);
          return nextRelatedSuggestions;
        })
        .catch(() => {
          setRelatedSuggestions([]);
          return [];
        });
    },
    [query, relatedLimit],
  );

  const reindex = useCallback(
    (nextDocuments: readonly TDocument[]) => {
      const provider = providerRef.current;
      if (!provider?.ready) return Promise.resolve();
      return provider.reindex(nextDocuments).then(() => {
        void search(query);
        if (enableRelated) void suggestRelated(query);
      });
    },
    [enableRelated, query, search, suggestRelated],
  );

  const clear = useCallback(() => {
    const provider = providerRef.current;
    if (!provider?.ready) return Promise.resolve();
    return provider.clear().then(() => {
      setResults([]);
      setSuggestions([]);
      setRelatedSuggestions([]);
    });
  }, []);

  useEffect(() => {
    let cancelled = false;
    const provider = new WorkerSearchProvider<TDocument>({
      documents,
      getText: options.getText,
      bm25: options.bm25,
      searchOptions: options.searchOptions,
      worker: options.worker,
      workerUrl: options.workerUrl,
      wasmModuleUrl: options.wasmModuleUrl,
    });

    providerRef.current = provider;
    setStatus('loading');
    setError(undefined);

    provider
      .init()
      .then(() => {
        if (cancelled) {
          provider.dispose();
          return;
        }
        setStatus('ready');
      })
      .catch((err: unknown) => {
        if (!cancelled) {
          setError(toError(err));
          setStatus('error');
        }
      });

    return () => {
      cancelled = true;
      provider.dispose();
      if (providerRef.current === provider) providerRef.current = undefined;
    };
  }, [documents, options]);

  useEffect(() => {
    if (status !== 'ready' || !autoSearch) return;
    void search(query);
    if (enableSuggest) void suggest(query);
    if (enableRelated) void suggestRelated(query);
  }, [autoSearch, enableRelated, enableSuggest, query, search, status, suggest, suggestRelated]);

  return {
    status,
    ready: status === 'ready',
    loading: status === 'loading',
    error,
    query,
    results,
    suggestions,
    relatedSuggestions,
    setQuery,
    search,
    suggest,
    suggestRelated,
    reindex,
    clear,
  };
}
