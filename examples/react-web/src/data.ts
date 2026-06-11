export type DemoDocument = {
  id: string;
  title: string;
  summary: string;
  content: string;
  route: string;
  tags: string[];
};

export const documents: DemoDocument[] = [
  {
    id: 'rust-backend',
    title: 'Rust 后端全文搜索',
    summary: '在 Rust 服务中直接引入 searchlight，构建倒排索引并返回高亮结果。',
    content:
      'Searchlight 支持中英文混合分词、BM25 排序、模糊搜索、拼音搜索和批量检索，适合作为后端搜索能力。',
    route: '/docs/rust-backend',
    tags: ['Rust', '后端', 'BM25'],
  },
  {
    id: 'react-hooks',
    title: 'React Hooks 本地搜索',
    summary: '通过 @luhanxin/searchlight-react 的 useSearchlight 直接管理 WASM 初始化、索引和搜索状态。',
    content:
      'useSearchlight 会返回 status、ready、query、setQuery、results、suggestions、search、reindex 和 clear，适合在 React 页面中快速接入本地搜索。',
    route: '/docs/react-hooks',
    tags: ['React', 'Hooks', 'WASM'],
  },
  {
    id: 'worker-hooks',
    title: 'Worker 搜索 Hook',
    summary: 'useSearchlightWorker 把索引和查询放到 Web Worker 中执行，避免大数据集阻塞主线程。',
    content:
      'WorkerSearchProvider 会在 Worker 内初始化 SearchEngine，并通过异步 search、suggest、reindex、clear 接口回传结果。',
    route: '/docs/worker-hooks',
    tags: ['Worker', '异步搜索', 'React'],
  },
  {
    id: 'pinyin-search',
    title: '拼音与首字母搜索',
    summary: '中文内容可以用 beijing、bj、bendissousuo 这类拼音或首字母进行检索。',
    content:
      '开启 usePinyin 后，Searchlight 会把中文词条映射到拼音索引，支持拼音前缀和首字母匹配。',
    route: '/docs/pinyin-search',
    tags: ['拼音', '中文', '搜索'],
  },
  {
    id: 'fuzzy-search',
    title: '模糊搜索与容错',
    summary: '输入拼写错误时，依然可以通过编辑距离找到相近词条。',
    content:
      '通过 fuzzy 和 maxEditDistance 可以开启模糊匹配，适合处理 programing、loacl search 这类拼写错误。',
    route: '/docs/fuzzy-search',
    tags: ['模糊搜索', '容错', '编辑距离'],
  },
  {
    id: 'explain-score',
    title: 'Explain 排序解释',
    summary: '开启 explain 后可以看到 BM25、覆盖度、短语命中、拼音命中等评分拆解。',
    content:
      'Searchlight 的结果可以返回 score_breakdown 和 match_reasons，用于调试搜索质量、解释排序原因以及展示命中依据。',
    route: '/docs/explain-score',
    tags: ['Explain', '排序解释', '调试'],
  },
];

export const workerDocuments: DemoDocument[] = [
  ...documents,
  {
    id: 'worker-reindex',
    title: 'Worker 动态重建索引',
    summary: '通过异步 reindex 把新的文档集合推送到 Worker，不需要阻塞 React 渲染。',
    content:
      '当文档来自远程接口或用户本地文件时，可以在 Worker 中重新批量索引，再立即使用 search 查询最新内容。',
    route: '/docs/worker-reindex',
    tags: ['Worker', 'reindex', '动态数据'],
  },
  {
    id: 'worker-suggest',
    title: 'Worker 拼音建议词',
    summary: 'suggest 同样可以在 Worker 中执行，输入 bj 或 bendissousuo 时返回候选词。',
    content:
      'useSearchlightWorker 的 suggest 返回 Promise，适合配合异步 UI 状态或防抖输入框展示搜索建议。',
    route: '/docs/worker-suggest',
    tags: ['Worker', 'suggest', '拼音建议'],
  },
  {
    id: 'worker-cache',
    title: 'Worker 查询缓存',
    summary: 'enableCache 可以复用热门查询结果，减少重复搜索开销。',
    content:
      '在 Worker 中开启缓存后，主线程只需要等待消息返回，适合高频输入、较大文档集和复杂排序解释场景。',
    route: '/docs/worker-cache',
    tags: ['Worker', '缓存', '性能'],
  },
];

export const localExamples = ['React Hooks', '本地搜索', 'bendissousuo', 'bj', 'programing', 'Explain'];

export const workerExamples = ['Worker', 'reindex', 'suggest', 'bj', '异步搜索', '缓存'];

export function toSearchText(document: DemoDocument) {
  return [document.title, document.summary, document.content, document.route, document.tags.join(' ')].join('\n');
}
