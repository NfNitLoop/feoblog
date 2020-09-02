import Vue from "vue/dist/vue.js" // The "dev" mode which can compile templates.
import bs58 from "bs58"
import commonmark from "commonmark"
import moment from "moment"

import { Item, Post } from "./protos/feoblog"

import * as nacl from "tweetnacl-ts"



// TODO:
const reader = new commonmark.Parser()
const writer = new commonmark.HtmlRenderer({ safe: true})
const MAX_ITEM_SIZE = 32 * 1024 // 32k


// Strictly parse one of these non-ambiguous timestamps. (MUST include time zone.)
const DATE_FORMATS = [
    // Preferred:
    "YYYY-MM-DD HH:mm:ss.SSS ZZ",
    // May drop milliseconds:
    "YYYY-MM-DD HH:mm:ss ZZ",
    // ... and seconds:
    "YYYY-MM-DD HH:mm ZZ",
]

function parseDate(str) {
    let date;
    for (let i in DATE_FORMATS) {
        // keep the parsed offset in the Moment so we can render/save it.
        date = moment.parseZone(str, DATE_FORMATS[i], true)
        if (date.isValid()) {
            return date
        }
    }
    return date;
}

// TODO: suppress warning about running in dev. mode. 
var app = new Vue({
    el: "#app",
    data: {
        title: "",
        post: "Hello, world!",
        timeInput_: "",
        // <3 Moment in that it'll keep the time and offset together:
        timestampMoment: moment(),

        userID: "",
        privateKey: "",
        signature: "",
        debug: false,
    },

    mounted: function() {
        let now = new Date();
        this.timeInput = moment().format(DATE_FORMATS[0])
        console.log("set timeInput:", this.timeInput)
        this.focusTextBox();
    },

    computed: {
        markdownOut: function() {
            var parsed = reader.parse(this.post);
            return writer.render(parsed);
        },
        
        // Used for display in the rendered post.
        // TODO: Fix for time offset.
        formattedDate: function() {
            if (!this.timestampUtcMs) {
                return "(invalid date)"
            }
            return new Date(this.timestampUtcMs).toISOString()
        },

        timestampUtcMs: function() {
            return this.timestampMoment.valueOf()
        },

        offsetMinutes: function() {
            return this.timestampMoment.utcOffset()
        },

        // Only updates timestamp 
        timeInput: {
            get: function() {
                return this.timeInput_;
            },
            set: function(newValue) {
                this.timeInput_ = newValue;

                let newDate = parseDate(newValue)
                if (!newDate.isValid()) {
                    return;
                }
                this.timestampMoment = newDate
            }
        },

        timeInputError: function() {
            let date = parseDate(this.timeInput_)
            if (!date.isValid()) {
                return "Invalid date format."
            }

            let now = moment()
            if (date.valueOf() > now.valueOf()) {
                return "Date is in the future."
            }

            return 
        },

        itemProto: function(): Item {
            let item = new Item({
                timestamp_ms_utc: this.timestampUtcMs,
                utc_offset_minutes: this.offsetMinutes,
                post: new Post()
            })

            // See: https://github.com/thesayyn/protoc-gen-ts/issues/16
            let post = item.post;
            if (this.title) { post.title = this.title }
            if (this.post) { post.body = this.post }
        

            return item;
        },

        itemProtoBytes: function() {
            return this.itemProto.serialize()
        },

        itemJson: function() {
            return JSON.stringify(this.itemProto.toObject(), null, 1)
        },

        protoSize: function() { 
            return this.itemProtoBytes.length
        },

        protoHex: function() {
            return bufferToHex(this.itemProtoBytes)
        },

        // This post is valid and signed and ready to send to the server:
        valid: function(): boolean {
            return this.errors.length == 0
        },

        errors: function() {
            return [... this.calculateErrors()]
        },

        validSignature: function(): boolean {
            if (!this.userID || !this.signature) {
                return false
            }
            try {
                let pubKey = bs58.decode(this.userID)
                let signature = bs58.decode(this.signature)
                let ok = nacl.sign_detached_verify(this.itemProtoBytes, signature, pubKey)
                return ok;
            } catch (error) {
                console.error("Error validating signature:", error)
                return false
            }
        }
        
    },

    watch: {
        privateKey: function(newValue) {
            console.debug(`Got pkey of length ${newValue.length}`)
            let buf;
            try {
                buf = bs58.decode(newValue)
            } catch (error) {
                console.error("Error during base58 decode", error)
                return
            }

            if (buf.length != 32) {
                console.debug("Want 32 bytes, but found", buf.length)
                return
            }
            
            // TODO: Consider using base58Check so that we can verify valid
            // pkeys if someone tries to type one in.

            let keypair = nacl.sign_keyPair_fromSeed(buf);
            this.userID = bs58.encode(keypair.publicKey)
            // TODO: sign the message.
            let signature = nacl.sign_detached(this.itemProtoBytes, keypair.secretKey)
            this.signature = bs58.encode(signature)

            // Delete the privateKey, we don't want to save it any longer than
            // necessary:
            this.privateKey=""
           
        },

        post: function(newValue) {
            if (newValue.startsWith("!!!debug")) {
                this.debug = true;
            }
        }
    },

    methods: {
        focusTextBox: function() {
            const box = this.$refs.textbox;
            box.focus();
            box.selectionStart = 0;
            box.selectionEnd = box.value.length;
        },

        calculateErrors: function*() {
            if (!this.userID) {
                yield "Must sign the message"
            }

            let itemLength = this.itemProtoBytes.length
            if (itemLength > MAX_ITEM_SIZE) {
                yield `Item size is ${itemLength}/${MAX_ITEM_SIZE}`
            }

            if (this.timeInputError) {
                yield this.timeInputError
            }

            // TODO: Check signature.
            if (!this.validSignature) {
                yield "Invalid Signature"
            }
        }
    }
});


function bufferToHex (x) {
    return [...new Uint8Array (x)]
        .map (b => b.toString(16).padStart(2, "0"))
        .join (" ");
}