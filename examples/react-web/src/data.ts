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
    id: 'react-wasm',
    title: 'React WASM 本地搜索',
    summary: '通过 wasm-bindgen 把 searchlight 打包成 npm 包，在 React/Vite 浏览器侧直接搜索。',
    content:
      'React 页面初始化 WebAssembly 后，可以把本地文档批量写入 SearchEngine，并使用 searchWithOptions 得到高亮结果。',
    route: '/docs/react-wasm',
    tags: ['React', 'WASM', '本地搜索'],
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
];

export function toSearchText(document: DemoDocument) {
  return [document.title, document.summary, document.content, document.tags.join(' ')].join('\n');
}
