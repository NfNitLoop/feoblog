import esbuild from "esbuild";
import sveltePlugin from "esbuild-svelte";
import sveltePreprocess from "svelte-preprocess";
import copy from 'esbuild-copy-plugin';
import inlineWorkerPlugin from 'esbuild-plugin-inline-worker';

import { execSync } from "child_process";

async function main() {

    // We usually don't need to constantly rebuild the protobuf code, so do that here
    // in case we're going to --watch:

    let watch = process.argv.includes("--watch")
    await doEsBuild({watch})
}



async function doEsBuild(opts = {}) {
    let result = await esbuild.build({
        entryPoints: ['index.js'],
        bundle: true,
        outfile: 'build/index.js',
        plugins: [
            inlineWorkerPlugin(),
            sveltePlugin({
                // Note: as of v4, this plugin does NOT check types!
                // That's handled by `npm run svelte-check` now.
                // See: https://github.com/sveltejs/svelte-preprocess/blob/main/docs/preprocessing.md#typescript---limitations
                preprocess: sveltePreprocess(),                
            }),
            copy({ from: "index.html", to: "index.html" }),
            copy({ from: "style.css", to: "style.css" }),
            copy({ from: "client.css", to: "client.css" }),
        ],
        define: {
            // See: https://github.com/evanw/esbuild/issues/73
            // Necessary for base58check -> readable-stream
            "global": "window"
        },
        format: "esm",
        minify: true,
        sourcemap: true,
        
        watch: opts.watch,
        logLevel: opts.watch ? "debug" : "info",
      }).catch(() => process.exit(1))    
}

await main()
