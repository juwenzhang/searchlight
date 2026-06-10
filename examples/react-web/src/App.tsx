import { useEffect, useMemo, useState } from 'react';
import { documents } from './data';
import { createSearchProvider, type SearchHit } from './search';

type SearchMode = 'local' | 'remote';

const examples = ['本地搜索', 'react wasm', 'bendissousuo', 'bj', 'programing'];

export default function App() {
  const [mode, setMode] = useState<SearchMode>('local');
  const [ready, setReady] = useState(false);
  const [query, setQuery] = useState('本地搜索');
  const [results, setResults] = useState<SearchHit[]>([]);
  const [suggestions, setSuggestions] = useState<string[]>([]);
  const [error, setError] = useState<string>();

  const provider = useMemo(() => createSearchProvider(mode), [mode]);

  function handleModeChange(nextMode: SearchMode) {
    setMode(nextMode);
    setReady(false);
    setResults([]);
    setSuggestions([]);
    setError(undefined);
  }

  useEffect(() => {
    let cancelled = false;

    Promise.resolve(provider.init?.())
      .then(() => {
        if (!cancelled) setReady(true);
      })
      .catch((err: unknown) => {
        if (!cancelled) setError(err instanceof Error ? err.message : String(err));
      });

    return () => {
      cancelled = true;
    };
  }, [provider]);

  useEffect(() => {
    let cancelled = false;

    if (!ready) return;

    provider
      .search(query)
      .then((items) => {
        if (!cancelled) setResults(items);
      })
      .catch((err: unknown) => {
        if (!cancelled) setError(err instanceof Error ? err.message : String(err));
      });

    provider
      .suggest?.(query)
      .then((items) => {
        if (!cancelled) setSuggestions(items);
      })
      .catch(() => {
        if (!cancelled) setSuggestions([]);
      });

    return () => {
      cancelled = true;
    };
  }, [provider, query, ready]);

  return (
    <main className="page-shell">
      <section className="hero-card">
        <div>
          <p className="eyebrow">Searchlight + React + WebAssembly</p>
          <h1>React 浏览器本地搜索示例</h1>
          <p className="hero-text">
            使用 Vite 官方 React 模板创建项目，再把 Rust 搜索引擎打包成 WASM npm 包，在浏览器侧完成全文搜索、拼音搜索、模糊搜索和高亮展示。
          </p>
        </div>

        <div className="mode-switch" aria-label="搜索模式">
          <button className={mode === 'local' ? 'active' : ''} onClick={() => handleModeChange('local')}>
            本地 WASM
          </button>
          <button className={mode === 'remote' ? 'active' : ''} onClick={() => handleModeChange('remote')}>
            远程 API
          </button>
        </div>
      </section>

      <section className="search-panel">
        <label htmlFor="query">搜索文档</label>
        <div className="search-row">
          <input
            id="query"
            value={query}
            disabled={!ready}
            placeholder="输入：本地搜索 / bendissousuo / bj / programing"
            onChange={(event) => setQuery(event.target.value)}
          />
          <span className={ready ? 'status ready' : 'status'}>{ready ? 'Ready' : 'Loading WASM'}</span>
        </div>

        <div className="examples">
          {examples.map((item) => (
            <button key={item} onClick={() => setQuery(item)}>
              {item}
            </button>
          ))}
        </div>

        {suggestions.length > 0 && (
          <div className="suggestions">
            <span>建议：</span>
            {suggestions.map((item) => (
              <button key={item} onClick={() => setQuery(item)}>
                {item}
              </button>
            ))}
          </div>
        )}

        {error && <p className="error">{error}</p>}
      </section>

      <section className="result-layout">
        <aside className="doc-list">
          <h2>已索引文档</h2>
          {documents.map((doc) => (
            <article key={doc.id}>
              <strong>{doc.title}</strong>
              <span>{doc.summary}</span>
            </article>
          ))}
        </aside>

        <section className="results">
          <h2>搜索结果</h2>
          {results.length === 0 ? (
            <p className="empty">暂无结果，试试 `bendissousuo`、`bj` 或 `programing`。</p>
          ) : (
            results.map((result) => (
              <article className="result-card" key={`${result.doc_id}-${result.score}`}>
                <div className="result-title">
                  <h3>{result.item?.title ?? `文档 ${result.doc_id}`}</h3>
                  <span>{result.score.toFixed(3)}</span>
                </div>
                <p dangerouslySetInnerHTML={{ __html: result.snippet ?? result.document }} />
                <div className="meta">
                  <span>{result.item?.route}</span>
                  <span>{result.matched_terms.join(', ') || '无匹配词'}</span>
                </div>
              </article>
            ))
          )}
        </section>
      </section>
    </main>
  );
}
