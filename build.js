const esbuild = require('esbuild');
const path = require('path');

const isWatch = process.argv.includes('--watch');

const buildOptions = {
  entryPoints: [
    'static/js/src/app.js',
    'static/js/src/theme.js',
  ],
  bundle: true,
  minify: !isWatch,
  sourcemap: isWatch,
  outdir: 'static/js/dist',
  format: 'iife',
  target: ['es2020'],
  logLevel: 'info',
};

async function build() {
  try {
    if (isWatch) {
      const ctx = await esbuild.context(buildOptions);
      await ctx.watch();
      console.log('Watching for changes...');
    } else {
      await esbuild.build(buildOptions);
      console.log('Build complete!');
    }
  } catch (error) {
    console.error('Build failed:', error);
    process.exit(1);
  }
}

build();
