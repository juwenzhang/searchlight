import { defineConfig } from 'tsup';

export default defineConfig({
  entry: {
    index: 'src/index.ts',
    'searchlight.worker': 'src/worker/searchlight.worker.ts',
  },
  format: ['esm', 'cjs'],
  target: 'es2020',
  dts: {
    compilerOptions: {
      ignoreDeprecations: '6.0',
    },
  },
  sourcemap: true,
  clean: true,
  minify: true,
  splitting: false,
  treeshake: true,
  external: ['react', '@luhanxin/searchlight'],
  outExtension({ format }) {
    return {
      js: format === 'cjs' ? '.cjs' : '.js',
    };
  },
});
