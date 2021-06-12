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
        <!--
            We've already recorded the audio, show a preview:
        -->
        <!-- svelte-ignore a11y-media-has-caption -->
        <AudioPlayer src={audioFileURL}/>
        <div class="buttons">
            <Button on:click={saveAudio}>Save</Button>
            <Button on:click={reset}>Reset</Button>
        </div>
    {:else}
        <div>
            <select bind:value={quality} disabled={isRecording || awaitingAudioDevice}>
                {#each qualityOptions as option (option)}
                    <option value={option}>{option.name}</option>
                {/each}
            </select>
        </div>
        <div class="buttons">
            <div class="inline" class:pulsing={isRecording}>
                <Button on:click={startRecording} disabled={isRecording || awaitingAudioDevice}>🔴 Record</Button>
            </div>
            <Button on:click={finishRecording} disabled={!isRecording}>⬛ Stop</Button>
        </div>
        {#if awaitingAudioDevice}
        <div class="warning">
            Waiting for permission to your microphone.
        </div>
        {/if}
    {/if}
    {#if isRecording || recordedBytes > 0}
    <div>
        Recorded: {readableSize(recordedBytes)}
    </div>
    {/if}
</div>

<script lang="ts" context="module">
import Button from "./Button.svelte"

// Use opus-media-recorder:
// See: https://github.com/kbumsik/opus-media-recorder#javascript
// Browser support for opus + containers is not good.
// * I found bugs recording directly to ogg/opus in FireFox. (Last seconds of audio were always lost)
// * Chrome only supoprts webm/opus
// * Safari only supports CAF/Opus (Core Audio File).
// * Firefox couldn't tell how long opus files were, but opus-media-recorder output works.
// So we'll use opus-media-recorder to provide consistent recording in all WASM-enabled browsers:

// Wikipedia says: 
// "Opus was originally specified for encapsulation in Ogg containers, specified as audio/ogg; codecs=opus, 
// and for Ogg Opus files the .opus filename extension is recommended."
const audioType = "audio/ogg; codecs=opus"
const fileExtension = ".opus"


// TODO: Separate decoder for playback:
// // For playback, this looks like a possibility:
// https://github.com/audiocogs/aurora.js


import OpusRecorder from 'opus-media-recorder';

const omrPath = "/client/opus-media-recorder"
const workerOptions = {
  // Note: OpusRecorder stops the worker when it's finished, so we must
  // construct a new one each time:
  encoderWorkerFactory: () => new Worker(`${omrPath}/encoderWorker.umd.js`),
  OggOpusEncoderWasmPath: `${omrPath}/OggOpusEncoder.wasm`,
};

export class AudioFile {
    name: string
    blob: Blob
}

</script>

<script lang="ts">
import { createEventDispatcher } from "svelte";
import { readableSize } from "../ts/common";
import AudioPlayer from "./AudioPlayer.svelte";


export let title = "Record Audio"
let error = ""
let stream: MediaStream|null = null
let recorder: any = null
let isRecording = false

// This state is enabled when we've asked the user for a Stream (microphone)
// but they haven't yet approved it.
let awaitingAudioDevice = false


type AudioQuality = {
    name: string,
    bitsPerSec: number,
    stereo: boolean
}

const qualityOptions: AudioQuality[] = [
    {
        name: "High Quality (Stereo, 128kb/s)",
        bitsPerSec: 128000,
        stereo: true,
    },
    {
        name: "High Quality (Mono, 64kb/s)",
        bitsPerSec: 64000,
        stereo: false,
    },
    {
        name: "Voice: High Quality (Mono, 32kb/s)",
        bitsPerSec: 32000,
        stereo: false,
    },
    {
        name: "Voice: Low Quality (Mono, 16kb/s)",
        bitsPerSec: 16000,
        stereo: false
    }
]

const hqStereo = qualityOptions[0]
const hqMono = qualityOptions[1]

// We default to hqStereo, but will fall back to mono if we detect mono.
let quality = hqStereo


// MediaRecorder will emit ondataavailable events and we need to collect those chunks:
// See: https://developer.mozilla.org/en-US/docs/Web/API/MediaRecorder/ondataavailable
let chunks: any[] = []

// Track recorded size so the user can see the size as they record:
let recordedBytes = 0


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
            error += "\nMost browsers require HTTPS to access the microphone."
        }
        return
    }
}

onLoad()

async function startRecording()
{
    console.debug("startRecording()")
    try {
        let timeslice = 1000;

        awaitingAudioDevice = true
        stream = await navigator.mediaDevices.getUserMedia({
            audio: {
                channelCount: { 
                    ideal: quality.stereo ? 2 : 1 
                },
            },
            video: false,
        })
        awaitingAudioDevice = false

        console.log(stream.id)
        stream.getAudioTracks().forEach((t) => {
            console.log("track:", t)
            console.log("label:", t.label)
            let s = t.getSettings()
            console.log("channels", s.channelCount)
            console.log("sampleRate:", s.sampleRate)
            console.log("settings:", s)
        })

        // Could do more generic fallback logic here, but for now hqStereo is the only stereo:
        if (quality == hqStereo && !isStereo(stream)) {
            console.log(`User selected ${quality.name}, but input is mono. Falling back to ${hqMono.name}.`)
            quality = hqMono
        }

        // TODO: Fall back to mono if the stream is mono.

        let options = { 
            mimeType: audioType,
            audioBitsPerSecond: quality.bitsPerSec
        }

        recorder = new OpusRecorder(stream, options, workerOptions)
        recorder.addEventListener("dataavailable", onDataAvailable)
        recorder.addEventListener("error", onMediaError)
        recorder.addEventListener("stop", onMediaStopped)

        chunks = []
        recordedBytes = 0
        recorder.start(timeslice)
        isRecording = true
    } catch (exception) {
        console.error(exception)
        error = `${exception}`
    }
}

function isStereo(stream: MediaStream): boolean {
    for (let t of stream.getAudioTracks()) {
        let channelCount = t?.getSettings()?.channelCount
        if (channelCount && channelCount >= 2) return true
    }
    return false
}

function finishRecording() {

    if (!stream) {
        console.error("finishRecording() called when stream is null.")
        return
    }

    recorder.stop()
    isRecording = false

    // TODO: Stop the stream too
    // 1) I've read this is REQUIRED on Safari.
    // 2) But this also tells the device/browser that we're not listening anymore.
    stream?.getTracks().forEach((t) => t.stop());
    stream = null
}

function reset() {
    audioFile = null
    audioFileURL = ""
    chunks = []
    recordedBytes = 0
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
    recordedBytes += data.size
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
    animation: 1000ms linear 0s infinite alternate both running pulsing;
}

.warning {
    background-color: lightyellow;
}

.warning:before {
    content: "⚠ ";
    color: orange;
}

@keyframes pulsing {
    from {
        opacity: 0.5;
    }
    to {
        opacity: 1;
    }
}
</style>