#![cfg(test)]

use crate::*;

#[tokio::test]
async fn simple_test() {
    let client = Client::new(ClientArgs{ 
        base_url: "https://blog.nfnitloop.com".to_owned()
    });

    let items = client.get_homepage(Default::default()).await.unwrap();

    println!("Got items: {items:#?}");

    assert!(items.get_items().len() > 0, "We should get some items");
}