import esbuild from "esbuild";
import sveltePlugin from "esbuild-svelte";
import sveltePreprocess from "svelte-preprocess";
import copy from 'esbuild-copy-plugin';
import inlineWorkerPlugin from 'esbuild-plugin-inline-worker';

main()
function main() {
    let watch = process.argv.includes("--watch")

    doEsBuild({watch})
}



function doEsBuild(opts = {}) {
    esbuild.build({
        entryPoints: ['index.js'],
        bundle: true,
        outfile: 'build/index.js',
        plugins: [
            inlineWorkerPlugin(),
            sveltePlugin({
                preprocess: sveltePreprocess(),
            }),
            copy({ from: "images/", to: "images/", }),
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
        
        watch: opts.watch,
        logLevel: opts.watch ? "debug" : "info",
      }).catch(() => process.exit(1))    
}

