use feo_client::{Client, ClientArgs, protobuf_types::ItemList, UserID, Signature, ItemResponse};
use leptos::{*, html::Input};
use leptos_meta::*;
use leptos_router::*;

use crate::helpers::Getter as _;

mod helpers;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/feoblog.css"/>

        // sets the document title
        <Title text="FeoBlog Leptos Placeholder"/>
        <Router>
            <nav>
                <a href="/leptos">"/leptos"</a>
                "    "
                <a href="/leptos/home">"/leptos/home"</a>
            </nav>
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
    let node = create_node_ref::<Input>();

    let _evt = create_effect(move |_| {
        node.get().map(|n| {
            n.select()
        });
    });
    view! {
        <p>"Hello, " {name.getter()} "!"</p>
        <input type="text"
            placeholder="What was your name, again?"
            prop:value={name.getter()}
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
    let load_items = create_blocking_resource(|| (), fetch_homepage);
    let items = move || {
        render_homepage(load_items.get())
    };

    view! {
        <Examples/>

        <Suspense>
            {items}
        </Suspense>
    }
}

async fn fetch_homepage(_: ()) -> Result<ItemList, ServerFnError> {
    let client = Client::new(ClientArgs{
        base_url: "https://blog.nfnitloop.com".into(),
    });
    let items = client.get_homepage(Default::default()).await?;
    Ok(items)
}

fn render_homepage(data: Option<Result<ItemList, ServerFnError>>) -> Result<View, ServerFnError> {
    let item_list = match data {
        Some(result) => result,
        None => return Ok(view! {"Loading ..."}.into_view())
    }?;

    let mut items: Vec<View> = vec![];
    for item in item_list.get_items().iter().take(10) {
        let uid = UserID::try_from(item.get_user_id().get_bytes())?;
        let sig = Signature::try_from(item.get_signature().get_bytes())?;
        // let get_uid = move || format!("{}", uid);
        // let get_sig = move || format!("{}", sig);
        // let item = view!{
        //     <article>
        //         {get_uid}
        //         <br/>
        //         {get_sig}
        //     </article>
        // };
        // items.push(item.into_view());
        let view = view! {
            <ItemView uid sig/>
        };
        items.push(view.into_view());
    }

    Ok(items.collect_view())

}

#[component]
fn ItemView(uid: UserID, sig: Signature) -> impl IntoView {
    let load_item = create_blocking_resource(
        move || (uid.clone(), sig.clone()),
        fetch_item
    );

    let item = move || render_item(load_item.get());

    view! {
        <Suspense>
            {item}
        </Suspense>
    }
}

async fn fetch_item(args: (UserID, Signature)) -> Result<ItemResponse, ServerFnError> {
    let (uid, sig) = args;
    let client = Client::new(ClientArgs{
        base_url: "https://blog.nfnitloop.com".into(),
    });
    let response = client.get_item(&uid, &sig).await?;
    Ok(response)
}

fn render_item(response: Option<Result<ItemResponse, ServerFnError>>) -> Result<View, ServerFnError> {
    let response = match response {
        Some(r) => r,
        None => return Ok(view! {
            "Loading..."
        }.into_view()),
    };
    let response = response?;
    let item = response.item;
    let body = item.get_post().get_body().to_owned();
    
    let view = view! {
        <article>
        <article-head>
            {response.user_id.to_string()} 
        </article-head>

        <article-body>
            <pre>{body}</pre>
        </article-body>
        </article>
    };

    Ok(view.into_view())
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