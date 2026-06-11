import type { DemoDocument } from '../data';

type DocumentShelfProps = {
  title: string;
  description: string;
  documents: DemoDocument[];
};

export function DocumentShelf({ title, description, documents }: DocumentShelfProps) {
  return (
    <aside className="panel doc-shelf">
      <div className="section-heading compact">
        <div>
          <p className="eyebrow">Indexed docs</p>
          <h2>{title}</h2>
          <p>{description}</p>
        </div>
        <span className="result-count">{documents.length} 篇</span>
      </div>

      <div className="doc-stack">
        {documents.map((doc) => (
          <article key={doc.id}>
            <strong>{doc.title}</strong>
            <span>{doc.summary}</span>
            <div className="tag-row">
              {doc.tags.map((tag) => (
                <span key={tag}>{tag}</span>
              ))}
            </div>
          </article>
        ))}
      </div>
    </aside>
  );
}
