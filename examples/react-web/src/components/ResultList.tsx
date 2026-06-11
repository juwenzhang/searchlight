import type { SearchlightHit } from '@luhanxin/searchlight-react';
import type { DemoDocument } from '../data';

type ResultListProps = {
  title: string;
  results: Array<SearchlightHit<DemoDocument>>;
  emptyText: string;
};

function formatScore(value: number) {
  return value.toFixed(3);
}

export function ResultList({ title, results, emptyText }: ResultListProps) {
  return (
    <section className="panel results-panel">
      <div className="section-heading compact">
        <div>
          <p className="eyebrow">Results</p>
          <h2>{title}</h2>
        </div>
        <span className="result-count">{results.length} 条</span>
      </div>

      {results.length === 0 ? (
        <p className="empty-state">{emptyText}</p>
      ) : (
        <div className="result-stack">
          {results.map((result) => (
            <article className="result-card" key={`${result.doc_id}-${result.score}`}>
              <div className="result-title">
                <div>
                  <h3>{result.item?.title ?? `文档 ${result.doc_id}`}</h3>
                  <p>{result.item?.summary}</p>
                </div>
                <span>{formatScore(result.score)}</span>
              </div>

              <p
                className="snippet"
                dangerouslySetInnerHTML={{ __html: result.snippet ?? result.document }}
              />

              <div className="meta-row">
                <span>{result.item?.route}</span>
                <span>{result.matched_terms.join(', ') || '无匹配词'}</span>
              </div>

              {result.score_breakdown && (
                <dl className="score-grid">
                  {Object.entries(result.score_breakdown).map(([key, value]) => (
                    <div key={key}>
                      <dt>{key}</dt>
                      <dd>{formatScore(value)}</dd>
                    </div>
                  ))}
                </dl>
              )}

              {result.match_reasons && result.match_reasons.length > 0 && (
                <ul className="reason-list">
                  {result.match_reasons.map((reason) => (
                    <li key={`${reason.code}-${reason.terms.join('-')}`}>
                      <strong>{reason.code}</strong>
                      <span>{reason.message}</span>
                    </li>
                  ))}
                </ul>
              )}
            </article>
          ))}
        </div>
      )}
    </section>
  );
}
