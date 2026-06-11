use std::cell::RefCell;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::num::NonZeroUsize;

use lru::LruCache;

use crate::{SearchOptions, SearchResult};

const SEARCH_RESULTS_CAPACITY: usize = 128;
const FUZZY_EXPANSION_CAPACITY: usize = 256;

/// LRU caches for repeated searches and fuzzy term expansion.
pub struct SearchCache {
    results: RefCell<LruCache<u64, Vec<SearchResult>>>,
    fuzzy_expansions: RefCell<LruCache<u64, Vec<String>>>,
}

impl SearchCache {
    pub fn new() -> Self {
        SearchCache {
            results: RefCell::new(LruCache::new(
                NonZeroUsize::new(SEARCH_RESULTS_CAPACITY).unwrap(),
            )),
            fuzzy_expansions: RefCell::new(LruCache::new(
                NonZeroUsize::new(FUZZY_EXPANSION_CAPACITY).unwrap(),
            )),
        }
    }

    pub fn clear(&self) {
        self.results.borrow_mut().clear();
        self.fuzzy_expansions.borrow_mut().clear();
    }

    pub fn get_results(&self, key: u64) -> Option<Vec<SearchResult>> {
        self.results.borrow_mut().get(&key).cloned()
    }

    pub fn put_results(&self, key: u64, results: Vec<SearchResult>) {
        self.results.borrow_mut().put(key, results);
    }

    pub fn get_fuzzy_expansion(&self, key: u64) -> Option<Vec<String>> {
        self.fuzzy_expansions.borrow_mut().get(&key).cloned()
    }

    pub fn put_fuzzy_expansion(&self, key: u64, terms: Vec<String>) {
        self.fuzzy_expansions.borrow_mut().put(key, terms);
    }
}

impl Default for SearchCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Cache key for a full search (query + options + index generation).
pub fn search_cache_key(generation: u64, query: &str, options: &SearchOptions) -> u64 {
    let mut hasher = DefaultHasher::new();
    generation.hash(&mut hasher);
    query.hash(&mut hasher);
    options.fuzzy.hash(&mut hasher);
    options.max_edit_distance.hash(&mut hasher);
    options.use_pinyin.hash(&mut hasher);
    options.highlight.hash(&mut hasher);
    options.limit.hash(&mut hasher);
    options.enable_cache.hash(&mut hasher);
    options.explain.hash(&mut hasher);
    hasher.finish()
}

/// Cache key for fuzzy term expansion.
pub fn fuzzy_cache_key(generation: u64, term: &str, max_distance: usize) -> u64 {
    let mut hasher = DefaultHasher::new();
    generation.hash(&mut hasher);
    term.hash(&mut hasher);
    max_distance.hash(&mut hasher);
    hasher.finish()
}
