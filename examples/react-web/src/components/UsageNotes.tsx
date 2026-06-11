type UsageNotesProps = {
  title: string;
  points: string[];
  code: string;
};

export function UsageNotes({ title, points, code }: UsageNotesProps) {
  return (
    <section className="panel usage-panel">
      <div className="section-heading compact">
        <div>
          <p className="eyebrow">Usage</p>
          <h2>{title}</h2>
        </div>
      </div>

      <ul>
        {points.map((point) => (
          <li key={point}>{point}</li>
        ))}
      </ul>

      <pre>
        <code>{code}</code>
      </pre>
    </section>
  );
}
