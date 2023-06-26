import esbuild from "esbuild";
import sveltePlugin from "esbuild-svelte";
import sveltePreprocess from "svelte-preprocess";
import copy from 'esbuild-copy-plugin';
import inlineWorkerPlugin from 'esbuild-plugin-inline-worker';
import fs from "node:fs";

async function main() {

    // We usually don't need to constantly rebuild the protobuf code, so do that here
    // in case we're going to --watch:

    let watch = process.argv.includes("--watch")
    await doEsBuild({watch})
}



async function doEsBuild(opts = {}) {
    let args = {
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
        metafile: true,
        
        logLevel: opts.watch ? "debug" : "info",
      }

    if (opts.watch) {
        const ctx = await esbuild.context(args)
        await ctx.watch()
        console.log("watching:")
    } else {
        const result = await esbuild.build(args)
        const json = JSON.stringify(result.metafile)
        fs.writeFileSync("esbuild.meta.json", json)

        // console.log(
        //     await esbuild.analyzeMetafile(result.metafile, {verbose: true})
        // )
    }

}

await main()
