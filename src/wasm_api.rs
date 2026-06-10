use crate::{SearchEngine as RustSearchEngine, SearchOptions, SearchResult};
use wasm_bindgen::prelude::*;

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct WasmSearchOptions {
    fuzzy: bool,
    max_edit_distance: usize,
    use_pinyin: bool,
    highlight: bool,
    limit: usize,
}

impl Default for WasmSearchOptions {
    fn default() -> Self {
        let options = SearchOptions::default();
        WasmSearchOptions {
            fuzzy: options.fuzzy,
            max_edit_distance: options.max_edit_distance,
            use_pinyin: options.use_pinyin,
            highlight: options.highlight,
            limit: options.limit,
        }
    }
}

impl From<WasmSearchOptions> for SearchOptions {
    fn from(options: WasmSearchOptions) -> Self {
        SearchOptions {
            fuzzy: options.fuzzy,
            max_edit_distance: options.max_edit_distance,
            use_pinyin: options.use_pinyin,
            highlight: options.highlight,
            limit: options.limit,
        }
    }
}

#[wasm_bindgen]
pub struct SearchEngine {
    inner: RustSearchEngine,
}

#[wasm_bindgen]
impl SearchEngine {
    #[wasm_bindgen(constructor)]
    pub fn new() -> SearchEngine {
        SearchEngine {
            inner: RustSearchEngine::new(),
        }
    }

    #[wasm_bindgen(js_name = withBm25Params)]
    pub fn with_bm25_params(k1: f64, b: f64) -> SearchEngine {
        SearchEngine {
            inner: RustSearchEngine::with_bm25_params(k1, b),
        }
    }

    pub fn index(&mut self, text: &str) -> usize {
        self.inner.index(text)
    }

    #[wasm_bindgen(js_name = indexBatch)]
    pub fn index_batch(&mut self, documents: JsValue) -> Result<JsValue, JsValue> {
        let documents: Vec<String> = from_js(documents, "indexBatch expects string[]")?;
        let ids = self.inner.index_batch(documents);
        to_js(&ids)
    }

    pub fn remove(&mut self, doc_id: usize) -> bool {
        self.inner.remove(doc_id)
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }

    #[wasm_bindgen(js_name = docCount)]
    pub fn doc_count(&self) -> usize {
        self.inner.doc_count()
    }

    #[wasm_bindgen(js_name = getDocument)]
    pub fn get_document(&self, doc_id: usize) -> Result<JsValue, JsValue> {
        let document = self.inner.get_document(doc_id).map(str::to_string);
        to_js(&document)
    }

    pub fn search(&self, query: &str) -> Result<JsValue, JsValue> {
        to_js(&self.inner.search(query))
    }

    #[wasm_bindgen(js_name = searchWithOptions)]
    pub fn search_with_options(&self, query: &str, options: JsValue) -> Result<JsValue, JsValue> {
        let options = options_from_js(options)?;
        to_js(&self.inner.search_with_options(query, &options))
    }

    #[wasm_bindgen(js_name = batchSearch)]
    pub fn batch_search(&self, queries: JsValue, options: JsValue) -> Result<JsValue, JsValue> {
        let queries: Vec<String> = from_js(queries, "batchSearch expects string[]")?;
        let options = options_from_js(options)?;
        let results: Vec<Vec<SearchResult>> = queries
            .iter()
            .map(|query| self.inner.search_with_options(query, &options))
            .collect();
        to_js(&results)
    }

    #[wasm_bindgen(js_name = searchPinyin)]
    pub fn search_pinyin(&self, query: &str) -> Result<JsValue, JsValue> {
        to_js(&self.inner.search_pinyin(query))
    }

    #[wasm_bindgen(js_name = searchFuzzy)]
    pub fn search_fuzzy(&self, query: &str, max_distance: usize) -> Result<JsValue, JsValue> {
        to_js(&self.inner.search_fuzzy(query, max_distance))
    }

    #[wasm_bindgen(js_name = searchPhrase)]
    pub fn search_phrase(&self, phrase: &str) -> Result<JsValue, JsValue> {
        to_js(&self.inner.search_phrase(phrase))
    }

    pub fn suggest(&self, prefix: &str) -> Result<JsValue, JsValue> {
        to_js(&self.inner.suggest(prefix))
    }

    #[wasm_bindgen(js_name = suggestWithPinyin)]
    pub fn suggest_with_pinyin(&self, prefix: &str) -> Result<JsValue, JsValue> {
        to_js(&self.inner.suggest_with_pinyin(prefix))
    }
}

fn options_from_js(value: JsValue) -> Result<SearchOptions, JsValue> {
    if value.is_undefined() || value.is_null() {
        return Ok(SearchOptions::default());
    }

    let options: WasmSearchOptions = from_js(value, "invalid search options")?;
    Ok(options.into())
}

fn from_js<T>(value: JsValue, context: &str) -> Result<T, JsValue>
where
    T: serde::de::DeserializeOwned,
{
    serde_wasm_bindgen::from_value(value)
        .map_err(|err| JsValue::from_str(&format!("{context}: {err}")))
}

fn to_js<T>(value: &T) -> Result<JsValue, JsValue>
where
    T: serde::Serialize,
{
    serde_wasm_bindgen::to_value(value)
        .map_err(|err| JsValue::from_str(&format!("failed to serialize value: {err}")))
}
