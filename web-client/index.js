import Vue from "vue/dist/vue.js" // The "dev" mode which can compile templates.
import bs58 from "bs58"
import commonmark from "commonmark"
import moment from "moment"


// TODO:
const reader = new commonmark.Parser()
const writer = new commonmark.HtmlRenderer({ safe: true})

// Strictly parse one of these non-ambiguous timestamps. (MUST include time zone.)
const DATE_FORMATS = [
    // Preferred:
    "YYYY-MM-DD hh:mm:ss.SSS ZZ",
    // May drop milliseconds:
    "YYYY-MM-DD hh:mm:ss ZZ",
    // ... and seconds:
    "YYYY-MM-DD hh:mm ZZ",
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

        // Used to generate the protobuf-encoded 
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
    },

    methods: {
        focusTextBox: function() {
            const box = this.$refs.textbox;
            box.focus();
            box.selectionStart = 0;
            box.selectionEnd = box.value.length;
        }
    }
});


