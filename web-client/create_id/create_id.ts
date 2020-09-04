import {html, customElement, property, LitElement} from "lit-element"

@customElement('create-id')
export class IDCreator extends LitElement {

    @property({type: Boolean}) bold = false;

    @property() text = "Default text"
    
    render() {
        return html`
        <style>
            .bold { font-weight: bold; }
            div { border: var(--create-id-div-border, none); }
        </style>
        <div class="${this.bold ? "bold" : ""}">
            <button @click="${this.click}">Click me</button>
            Hello! Bold: ${this.bold}
        <div>
        <div>
            <input type="text" value="${this.text}" @change="${this.updateText}">
            <br>${this.text}
        </div>
        `
    }

    click() {
        this.bold = !this.bold;
    }

    updateText(e, f, g) {
        let target = e.currentTarget
        this.text = target.value
        console.log("got here")
    }
}

console.log("got here")