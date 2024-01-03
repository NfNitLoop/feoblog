use feo_client::{Client, ClientArgs, protobuf_types::ItemList};
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
                <Route path="/leptos/" view=Greeter/>
                <Route path="/leptos/home" view=Home/>
            </Routes>
        </Router>
    }
}

#[component]
pub fn Greeter() -> impl IntoView {
    let name = create_rw_signal(String::new());
    let get_name = move || name.get();
    let node = create_node_ref::<Input>();

    let _evt = create_effect(move |_| {
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
        <div>
            <A href="/leptos/home">Home</A>
        </div>
    }
}

#[component]
pub fn Home() -> impl IntoView {


    async fn fetcher(_: ()) -> Result<ItemList, ServerFnError> {
        let client = Client::new(ClientArgs{
            base_url: "https://blog.nfnitloop.com".into(),
        });
        let items = client.get_homepage(Default::default()).await?;
        Ok(items)
    }

    let rsc = create_blocking_resource(|| (), fetcher);
    let items = move || {
        rsc.get().map(|items| {
            let output = format!("{items:#?}");
            view! {
                <round-box>
                    <pre>{output}</pre>
                </round-box>
            }
        })
    };

    view! {
        <Examples/>

        <Suspense fallback=move || view! { <p>"Loading ..."</p> }>
            {items}
        </Suspense>
    }
}

#[component]
fn Examples() -> impl IntoView {
    view! {
        <article>
            <article-head>
                <img
                    class="profileImage"
                    src="/u/C1KxN5L4tYjEv6vkxjD8mMD917ntKTG5r1vKYmhJgyQX/icon.png"
                    alt="identicon"/>

                    <a class="userID" 
                        href="#/u/C1KxN5L4tYjEv6vkxjD8mMD917ntKTG5r1vKYmhJgyQX/" 
                        title="C1KxN5L4tYjEv6vkxjD8mMD917ntKTG5r1vKYmhJgyQX"
                    >"Mastodon Feed"</a>
                    
                    <a class="timestamp" 
                        href="#/u/C1KxN5L4tYjEv6vkxjD8mMD917ntKTG5r1vKYmhJgyQX/i/4AxdtMxv5U9kbXDYT3PyneEH4Xn7k8Wr3DGYaAFF5FVjDVR894E2S7K1WXpkSaeqwyfyvPomjeKi5aRq764zvmpx/" 
                        title="2024-01-01 23:19:10 +0000\n2024-01-01 15:19:10 -0800">"3 hours ago"</a>

                <Arrow/>
            </article-head>
            <article-body>
                <p>"Lorum ipsum dolor sit amet ..."</p>
            </article-body>
        </article>

        <round-box>
            <h1>Heading</h1>
            <p>"Here's a sample paragraph."</p>
        </round-box>
    }
}

#[component]
pub fn Arrow() -> impl IntoView {
    view! {
        <svg class="openArrow left" viewBox="0 0 200 200" fill="none" xmlns="http://www.w3.org/2000/svg">
            <path d="M67 34L133 100L67 166" stroke="black" stroke-width="13" stroke-linecap="round" class="svelte-vh3cio"></path>
        </svg>
    }
}