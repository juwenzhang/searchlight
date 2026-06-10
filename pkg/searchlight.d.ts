/* tslint:disable */
/* eslint-disable */

export class SearchEngine {
    free(): void;
    [Symbol.dispose](): void;
    batchSearch(queries: any, options: any): any;
    clear(): void;
    docCount(): number;
    getDocument(doc_id: number): any;
    index(text: string): number;
    indexBatch(documents: any): any;
    constructor();
    remove(doc_id: number): boolean;
    search(query: string): any;
    searchFuzzy(query: string, max_distance: number): any;
    searchPhrase(phrase: string): any;
    searchPinyin(query: string): any;
    searchWithOptions(query: string, options: any): any;
    suggest(prefix: string): any;
    suggestWithPinyin(prefix: string): any;
    static withBm25Params(k1: number, b: number): SearchEngine;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_searchengine_free: (a: number, b: number) => void;
    readonly searchengine_batchSearch: (a: number, b: any, c: any) => [number, number, number];
    readonly searchengine_clear: (a: number) => void;
    readonly searchengine_docCount: (a: number) => number;
    readonly searchengine_getDocument: (a: number, b: number) => [number, number, number];
    readonly searchengine_index: (a: number, b: number, c: number) => number;
    readonly searchengine_indexBatch: (a: number, b: any) => [number, number, number];
    readonly searchengine_new: () => number;
    readonly searchengine_remove: (a: number, b: number) => number;
    readonly searchengine_search: (a: number, b: number, c: number) => [number, number, number];
    readonly searchengine_searchFuzzy: (a: number, b: number, c: number, d: number) => [number, number, number];
    readonly searchengine_searchPhrase: (a: number, b: number, c: number) => [number, number, number];
    readonly searchengine_searchPinyin: (a: number, b: number, c: number) => [number, number, number];
    readonly searchengine_searchWithOptions: (a: number, b: number, c: number, d: any) => [number, number, number];
    readonly searchengine_suggest: (a: number, b: number, c: number) => [number, number, number];
    readonly searchengine_suggestWithPinyin: (a: number, b: number, c: number) => [number, number, number];
    readonly searchengine_withBm25Params: (a: number, b: number) => number;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_exn_store: (a: number) => void;
    readonly __externref_table_alloc: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
