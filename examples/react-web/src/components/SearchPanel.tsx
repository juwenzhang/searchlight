import type { SearchlightRelatedSuggestion, SearchlightStatus } from '@luhanxin/searchlight-react';

type SearchPanelProps = {
  title: string;
  description: string;
  query: string;
  ready: boolean;
  status: SearchlightStatus;
  placeholder: string;
  examples: string[];
  suggestions: string[];
  relatedSuggestions?: SearchlightRelatedSuggestion[];
  error?: Error;
  actionNote?: string;
  onQueryChange(query: string): void;
  onSearch?: () => unknown;
  onClear?: () => unknown;
};

function statusText(status: SearchlightStatus) {
  if (status === 'ready') return 'Ready';
  if (status === 'error') return 'Error';
  if (status === 'loading') return 'Loading';
  return 'Idle';
}

export function SearchPanel({
  title,
  description,
  query,
  ready,
  status,
  placeholder,
  examples,
  suggestions,
  relatedSuggestions = [],
  error,
  actionNote,
  onQueryChange,
  onSearch,
  onClear,
}: SearchPanelProps) {
  return (
    <section className="panel search-panel">
      <div className="section-heading">
        <div>
          <p className="eyebrow">Hook usage</p>
          <h2>{title}</h2>
          <p>{description}</p>
        </div>
        <span className={`status status-${status}`}>{statusText(status)}</span>
      </div>

      <div className="search-row">
        <input
          value={query}
          disabled={!ready}
          placeholder={placeholder}
          onChange={(event) => onQueryChange(event.target.value)}
        />
        {onSearch && (
          <button className="primary-button" disabled={!ready} onClick={() => void onSearch()}>
            搜索
          </button>
        )}
        {onClear && (
          <button className="ghost-button" disabled={!ready} onClick={() => void onClear()}>
            清空
          </button>
        )}
      </div>

      <div className="chips" aria-label="示例查询">
        {examples.map((item) => (
          <button key={item} onClick={() => onQueryChange(item)}>
            {item}
          </button>
        ))}
      </div>

      {suggestions.length > 0 && (
        <div className="suggestions">
          <span>补全建议</span>
          {suggestions.map((item) => (
            <button key={item} onClick={() => onQueryChange(item)}>
              {item}
            </button>
          ))}
        </div>
      )}

      {relatedSuggestions.length > 0 && (
        <div className="suggestions related-suggestions">
          <span>相关提示词</span>
          {relatedSuggestions.map((item) => (
            <button key={item.term} title={`score ${item.score.toFixed(3)}`} onClick={() => onQueryChange(item.term)}>
              {item.term}
            </button>
          ))}
        </div>
      )}

      {actionNote && <p className="action-note">{actionNote}</p>}
      {error && <p className="error-message">{error.message}</p>}
    </section>
  );
}
