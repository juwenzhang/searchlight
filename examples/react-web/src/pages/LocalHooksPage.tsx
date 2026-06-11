import { useSearchlight } from '@luhanxin/searchlight-react';
import { DocumentShelf } from '../components/DocumentShelf';
import { ResultList } from '../components/ResultList';
import { SearchPanel } from '../components/SearchPanel';
import { UsageNotes } from '../components/UsageNotes';
import { documents, localExamples, toSearchText } from '../data';

const localHookOptions = {
  initialQuery: 'React Hooks',
  getText: toSearchText,
  relatedLimit: 8,
  searchOptions: {
    fuzzy: true,
    maxEditDistance: 2,
    usePinyin: true,
    highlight: true,
    limit: 8,
    enableCache: true,
    explain: true,
  },
};

const localHookCode = `const search = useSearchlight(documents, {
  initialQuery: 'React Hooks',
  getText: toSearchText,
  searchOptions: { fuzzy: true, usePinyin: true, explain: true },
});`;

export function LocalHooksPage() {
  const search = useSearchlight(documents, localHookOptions);

  return (
    <div className="demo-page">
      <SearchPanel
        title="useSearchlight：主线程本地搜索"
        description="Hook 负责初始化 WASM、创建 SearchEngine、索引文档，并在 query 变化时自动搜索和生成建议词。"
        query={search.query}
        ready={search.ready}
        status={search.status}
        placeholder="输入：React Hooks / bendissousuo / bj / programing"
        examples={localExamples}
        suggestions={search.suggestions}
        relatedSuggestions={search.relatedSuggestions}
        error={search.error}
        onQueryChange={search.setQuery}
        onSearch={() => search.search(search.query)}
        onClear={search.clear}
      />

      <div className="content-grid">
        <DocumentShelf
          title="默认本地索引"
          description="这些文档会在浏览器主线程中写入 WASM SearchEngine。"
          documents={documents}
        />
        <ResultList
          title="本地 Hook 搜索结果"
          results={search.results}
          emptyText="暂无结果，试试 bj、bendissousuo 或 programing。"
        />
      </div>

      <UsageNotes
        title="本地 Hook 关键点"
        points={[
          'options 放在组件外保持引用稳定，避免重复初始化索引。',
          'results 已自动挂载原始 document，可直接读取 item.title、item.route。',
          '开启 explain 后，结果会展示 score_breakdown 与 match_reasons。',
          'relatedSuggestions 来自 Rust 核心 suggestRelated，只做候选召回和排序，不做生成。',
        ]}
        code={localHookCode}
      />
    </div>
  );
}
