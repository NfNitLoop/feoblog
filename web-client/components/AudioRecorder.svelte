<!--
    Records audio. 
    Emits a "savedAudio" event with the audio blob when it's finished.
    The .data for that is an AudioFile instance.
-->
<div>
    <h2>{title}</h2>

    <!-- TODO: Add a stage where you can select the recording bitrate? -->
    <!--
        TODO:
        Warn that audio recording is not currently supported on iOS. :( 
        See:
        https://stackoverflow.com/questions/42221646/voice-recording-on-iphone-by-using-safari-and-html5
    -->
    {#if error}
    <div class="error">
        {error}
    </div>
    {:else if audioFileURL}
        <!-- svelte-ignore a11y-media-has-caption -->
        <audio kind="audio" controls src={audioFileURL}/>
        <div class="buttons">
            <Button on:click={saveAudio}>Save</Button>
            <Button on:click={reset}>Reset</Button>
        </div>
    {:else if !recorder} 
    <div>
        ⚠ You must first allow access to your microphone.
    </div>
    {:else}
    <div class="buttons">
        {#if !isRecording}
            <Button on:click={startRecording}>⏺ Record</Button>
        {:else}
            <div class="inline pulsing">Recording ...</div>
            <Button on:click={finishRecording}>⏹ Stop</Button>
        {/if}
    </div>
    {/if}
    {#each supportedTypes as sType (sType)}
        {sType}<br/>
    {/each}


</div>

<script lang="ts" context="module">
import Button from "./Button.svelte"

// See: https://github.com/microsoft/TypeScript/issues/34728
// Using the third-party types didn't work for me. (Do I need to import them somehow?)
declare var MediaRecorder: any;

// Wikipedia says: 
// "Opus was originally specified for encapsulation in Ogg containers, specified as audio/ogg; codecs=opus, 
// and for Ogg Opus files the .opus filename extension is recommended."
// const audioType = "audio/ogg; codecs=opus"
// const fileExtension = ".opus"
// However, this is only supported in FireFox.
// Additionally, when I tried it, the last few seconds of recording would always get cut off,
// but that issue does not happen when I use the webm container.

const audioType = "audio/webm; codecs=opus"
// Technically, only the .webm container is required, but that's an ambiguous type.
// Let's make it explicit that this is just a single Opus audio stream encoded in WebM:
const fileExtension = ".opus.webm"

const knownAudioTypes = [
    "audio/webm; codecs=opus",
    "audio/webm;codecs=opus",
    "audio/mp4",
    "audio/mp3",

    "audio/wave",
    "audio/wav",
    "audio/ogg",
    "audio/webm",
    "audio/ogg;codecs=opus",
]

// TODO: Use https://jsfiddle.net/kbumsik/v3wpnxao/
// See: https://github.com/kbumsik/opus-media-recorder#javascript
// Browser support for opus + containers is not good.
// * I found bugs recording directly to ogg/opus in FireFox. (Last seconds of audio were always lost)
// * Chrome only supoprts webm/opus
// * Safari only supports CAF/Opus (Core Audio File).
// So we'll use opus-media-recorder to provide consistent recording in all WASM-enabled browsers:
// https://github.com/kbumsik/opus-media-recorder#javascript
// I prefer ogg/opus since Opus was supposedly designed with that in mind.
// Plus, that lets us use the unambiguous .opus file extension.
// TODO:


// TODO: Separate decoder for playback:
// // For playback, this looks like a 
// https://github.com/audiocogs/aurora.js


// import OMRecorder from 'opus-media-recorder';
// // Use worker-loader
// import EncoderWorker from 'worker-loader!opus-media-recorder/encoderWorker.js';
// // You should use file-loader in webpack.config.js.
// // See webpack example link in the above section for more detail.
// import OggOpusWasm from 'opus-media-recorder/OggOpusEncoder.wasm';
// import WebMOpusWasm from 'opus-media-recorder/WebMOpusEncoder.wasm';

// // Non-standard options
// const workerOptions = {
//   encoderWorkerFactory: _ => new EncoderWorker(),
//   OggOpusEncoderWasmPath: OggOpusWasm,
//   WebMOpusEncoderWasmPath: WebMOpusWasm
// };


export class AudioFile {
    name: string
    blob: Blob
}

</script>

<script lang="ts">
import { createEventDispatcher } from "svelte";


export let title = "Record Audio"
let error = ""
let stream: MediaStream|null = null
let recorder: any = null
let isRecording = false

// Calculated from knownAudioTypes in onLoad():
let supportedTypes: string[] = []


// MediaRecorder will emit ondataavailable events and we need to collect those chunks:
// See: https://developer.mozilla.org/en-US/docs/Web/API/MediaRecorder/ondataavailable
let chunks: any[] = []

// Only after we have all chunks do we collect them into a blob:
let audioFile: Blob|null = null
// An objectURL pointing to the audioFile Blob:
let audioFileURL: string = ""

let dispatcher = createEventDispatcher()

async function onLoad() {
    let deviceSupportsRecording = navigator?.mediaDevices?.getUserMedia != null
    if (!deviceSupportsRecording) {
        error = "This browser/device does not support audio recording."

        if (!window.location.toString().startsWith("https://")) {
            error += "\nSome browsers (like Safari) require HTTPS access to use MediaRecorder."
        }
        return
    }

    let isSupported: (t: string) => boolean = MediaRecorder.isTypeSupported;
    supportedTypes = knownAudioTypes.filter(isSupported);

    if (!isSupported(audioType)) {
        error = `Your browser does not support the audio recording format: ${audioType}`
        return
    }

    try {
        stream = await navigator.mediaDevices.getUserMedia({audio: true, video: false})
        console.debug(`Got media stream: ${stream}`)

        let options = { mimeType: audioType }

        recorder = new MediaRecorder(stream, options)
        recorder.addEventListener("dataavailable", onDataAvailable)
        recorder.addEventListener("error", onMediaError)
        recorder.addEventListener("stop", onMediaStopped)

    } catch (error) {
        console.error(error)
    }
}

onLoad()

function startRecording()
{
    // Will this keep things from getting as truncated?
    let timeslice = undefined;
    recorder.start(timeslice)
    isRecording = true
}

function finishRecording() {

    if (!stream) {
        console.error("finishRecording() called when stream is null.")
        return
    }

    // If we stop the *recorder* when recording an Ogg Opus (.opus), it seems like we miss the last part of the stream.
    // Instead, stop the *stream* and let the recorder end when it gets the last of the stream:
    // recorder.stop()
    // Nope, that's not it.
    stream.getTracks().forEach((t) => t.stop());

    isRecording = false
}

function reset() {
    audioFile = null
    audioFileURL = ""
    chunks = []
}

// The user has had the option to listen to the audio and has clicked "save".
function saveAudio() {

    if (!audioFile) {
        console.error("Tried to saveAudio() when there is no audioFile")
        return
    }

    let info: AudioFile = {
        name: `recording${fileExtension}`,
        blob: audioFile,
    };
    dispatcher("savedAudio", info)

}

function onDataAvailable(event: any) {
    console.debug("onDataAvailable()")
    let data: Blob = event.data
    chunks.push(data)
}

function onMediaStopped() {
    console.debug("onMediaStopped()")
    audioFile = new Blob(chunks, {type: audioType})
    audioFileURL = URL.createObjectURL(audioFile)
    chunks = []
    isRecording = false
}

function onMediaError(e: any) {
    error = `${e}`
}

// TODO: Stop & clean up recording if the widget is unloaded.
// TODO: Clean up URL.createObjectURL().
</script>

<style>
.buttons {
    margin-top: 0.5rem;
    margin-bottom: 0.5rem;
}

.inline {
    display: inline-block;
}

.pulsing {
    animation: 600ms linear 0s infinite alternate both running pulsing;
}

@keyframes pulsing {
    from {
        opacity: 0;
    }
    to {
        opacity: 1;
    }
}
</style>