use searchlight::{SearchEngine, SearchOptions};

fn main() {
    let mut engine = SearchEngine::new();
    engine.index_batch([
        "Rust 是一门现代系统编程语言，安全且高效",
        "React 可以结合 WASM 实现浏览器本地搜索",
        "拼音搜索支持 beijing、bj 这类输入",
    ]);

    let options = SearchOptions {
        fuzzy: true,
        max_edit_distance: 2,
        use_pinyin: true,
        highlight: true,
        limit: 10,
    };

    for result in engine.search_with_options("bendi sousuo", &options) {
        println!(
            "doc={} score={:.3} text={}",
            result.doc_id,
            result.score,
            result.snippet.as_deref().unwrap_or(&result.document)
        );
    }
}
