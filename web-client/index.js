import Vue from "vue/dist/vue.js" // The "dev" mode which can compile templates.
import bs58 from "bs58"
import commonmark from "commonmark"


// TODO:
// * Disable inline HTML blocks.
// * Replace custom links.
const reader = new commonmark.Parser();
const writer = new commonmark.HtmlRenderer({ safe: true});



var app = new Vue({
    el: "#app",
    data: {
        title: "My Post",
        post: "Hello, world!",
        timestamp: new Date(),
    },
    computed: {
        markdownOut: function() {
            var parsed = reader.parse(this.post);
            return writer.render(parsed);
            console.log("got here");
        }    
    },
    mounted: function() {
        this.focusTextBox();
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


