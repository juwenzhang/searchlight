import { useCallback, useEffect, useRef, useState } from 'react';
import { createSearchlightEngine } from './core';
import { LocalSearchProvider } from './provider';
import type {
  SearchEngine,
  SearchlightHit,
  SearchlightSearchOptions,
  SearchlightStatus,
  UseSearchlightEngineState,
  UseSearchlightOptions,
  UseSearchlightState,
} from './types';

function toError(error: unknown) {
  return error instanceof Error ? error : new Error(String(error));
}

export function useSearchlightEngine(options: Pick<UseSearchlightOptions, 'wasmModule' | 'bm25'> = {}): UseSearchlightEngineState {
  const [status, setStatus] = useState<SearchlightStatus>('idle');
  const [engine, setEngine] = useState<SearchEngine>();
  const [error, setError] = useState<Error>();

  useEffect(() => {
    let cancelled = false;
    let nextEngine: SearchEngine | undefined;

    setStatus('loading');
    setError(undefined);

    createSearchlightEngine(options)
      .then((createdEngine) => {
        nextEngine = createdEngine;
        if (cancelled) {
          createdEngine.free();
          return;
        }
        setEngine(createdEngine);
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
      nextEngine?.free();
      setEngine(undefined);
    };
  }, [options]);

  return {
    status,
    ready: status === 'ready',
    loading: status === 'loading',
    error,
    engine,
  };
}

export function useSearchlight<TDocument = string>(
  documents: readonly TDocument[],
  options: UseSearchlightOptions<TDocument> = {},
): UseSearchlightState<TDocument> {
  const providerRef = useRef<LocalSearchProvider<TDocument> | undefined>(undefined);
  const [status, setStatus] = useState<SearchlightStatus>('idle');
  const [error, setError] = useState<Error>();
  const [query, setQuery] = useState(options.initialQuery ?? '');
  const [results, setResults] = useState<Array<SearchlightHit<TDocument>>>([]);
  const [suggestions, setSuggestions] = useState<string[]>([]);
  const autoSearch = options.autoSearch ?? true;
  const enableSuggest = options.suggest ?? true;

  const search = useCallback(
    (nextQuery = query, searchOptions?: SearchlightSearchOptions) => {
      const provider = providerRef.current;
      if (!provider?.ready) return [];

      try {
        const nextResults = provider.search(nextQuery, searchOptions);
        setResults(nextResults);
        setError(undefined);
        return nextResults;
      } catch (err: unknown) {
        setError(toError(err));
        setStatus('error');
        return [];
      }
    },
    [query],
  );

  const suggest = useCallback(
    (prefix = query) => {
      const provider = providerRef.current;
      if (!provider?.ready) return [];

      try {
        const nextSuggestions = provider.suggest(prefix);
        setSuggestions(nextSuggestions);
        return nextSuggestions;
      } catch {
        setSuggestions([]);
        return [];
      }
    },
    [query],
  );

  const reindex = useCallback(
    (nextDocuments: readonly TDocument[]) => {
      const provider = providerRef.current;
      if (!provider?.ready) return;
      provider.reindex(nextDocuments);
      search(query);
    },
    [query, search],
  );

  const clear = useCallback(() => {
    const provider = providerRef.current;
    if (!provider?.ready) return;
    provider.clear();
    setResults([]);
    setSuggestions([]);
  }, []);

  useEffect(() => {
    let cancelled = false;
    const provider = new LocalSearchProvider<TDocument>({
      documents,
      getText: options.getText,
      wasmModule: options.wasmModule,
      bm25: options.bm25,
      searchOptions: options.searchOptions,
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
    search(query);
    if (enableSuggest) suggest(query);
  }, [autoSearch, enableSuggest, query, search, status, suggest]);

  return {
    status,
    ready: status === 'ready',
    loading: status === 'loading',
    error,
    query,
    results,
    suggestions,
    setQuery,
    search,
    suggest,
    reindex,
    clear,
  };
}
