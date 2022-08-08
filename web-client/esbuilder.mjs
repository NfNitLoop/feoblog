import esbuild from "esbuild";
import sveltePlugin from "esbuild-svelte";
import sveltePreprocess from "svelte-preprocess";
import copy from 'esbuild-copy-plugin';
import inlineWorkerPlugin from 'esbuild-plugin-inline-worker';

import { execSync } from "child_process";

main()
function main() {

    // We usually don't need to constantly rebuild the protobuf code, so do that here
    // in case we're going to --watch:
    protoToTypeScript()

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

function protoToTypeScript() {
    let cmd =
        "protoc"
        + ` --plugin=./node_modules/.bin/${npmScript('protoc-gen-ts')}`
        + " --ts_out=protos"
        + " --proto_path=../protobufs/"
        + " feoblog.proto"
    ;

    console.log("Running", cmd);

    execSync(cmd, (error, stdout, stderr) => {
        if (error) {
            console.log("Error building proto file: ", e);
           throw error;
        }
    });
    console.log("Done")
}

function npmScript(name) {
    if (isWindows()) {
        return `${name}.cmd`
    }
    return name
}

function isWindows() {
    return process.platform == "win32"
}