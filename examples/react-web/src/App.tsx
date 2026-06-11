import { useState } from 'react';
import { LocalHooksPage } from './pages/LocalHooksPage';
import { WorkerHooksPage } from './pages/WorkerHooksPage';

type DemoPage = 'hooks' | 'worker';

const pages: Array<{ id: DemoPage; label: string; description: string }> = [
  {
    id: 'hooks',
    label: 'useSearchlight',
    description: '主线程 WASM Hook',
  },
  {
    id: 'worker',
    label: 'useSearchlightWorker',
    description: 'Web Worker 异步 Hook',
  },
];

export default function App() {
  const [activePage, setActivePage] = useState<DemoPage>('hooks');

  return (
    <main className="app-shell">
      <section className="hero-card">
        <div>
          <p className="eyebrow">Searchlight React Hooks Demo</p>
          <h1>用真实 hooks 演示浏览器搜索能力</h1>
          <p className="hero-text">
            示例应用已从手写 provider 迁移到 <code>@luhanxin/searchlight-react</code>，分别演示
            <code>useSearchlight</code> 与新增的 <code>useSearchlightWorker</code>。
          </p>
        </div>

        <nav className="page-tabs" aria-label="示例页面">
          {pages.map((page) => (
            <button
              key={page.id}
              className={activePage === page.id ? 'active' : ''}
              onClick={() => setActivePage(page.id)}
            >
              <strong>{page.label}</strong>
              <span>{page.description}</span>
            </button>
          ))}
        </nav>
      </section>

      {activePage === 'hooks' ? <LocalHooksPage /> : <WorkerHooksPage />}
    </main>
  );
}
