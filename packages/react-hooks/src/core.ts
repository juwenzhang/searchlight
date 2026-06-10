import init, { SearchEngine } from '@luhanxin/searchlight';
import type { InitInput, SearchlightBm25Options } from './types';

let wasmReady: Promise<void> | undefined;

export function initSearchlight(wasmModule?: InitInput | Promise<InitInput>) {
  wasmReady ??= init(wasmModule).then(() => undefined);
  return wasmReady;
}

export async function createSearchlightEngine(options: { wasmModule?: InitInput | Promise<InitInput>; bm25?: SearchlightBm25Options } = {}) {
  await initSearchlight(options.wasmModule);
  return options.bm25 ? SearchEngine.withBm25Params(options.bm25.k1, options.bm25.b) : new SearchEngine();
}
