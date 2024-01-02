use leptos::{*, html::Input};
use leptos_meta::*;
use leptos_router::*;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/feoblog.css"/>

        // sets the document title
        <Title text="FeoBlog Leptos Placeholder"/>
        <Router>
            <Routes>
                <Route path="/leptos" view=Greeter/>
            </Routes>
        </Router>
    }
}

#[component]
pub fn Greeter() -> impl IntoView {
    let name = create_rw_signal(String::new());
    let get_name = move || name.get();
    let node = create_node_ref::<Input>();

    let evt = create_effect(move |_| {
        node.get().map(|n| {
            n.select()
        });
    });
    view! {
        <p>"Hello, " {get_name} "!"</p>
        <input type="text"
            placeholder="What was your name, again?"
            prop:value={get_name}
            on:input=move |ev| {
                name.set(event_target_value(&ev));
            }
            node_ref=node
        />
    }
}