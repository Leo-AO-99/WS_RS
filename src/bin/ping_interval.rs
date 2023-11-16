use futures_util::StreamExt;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_util::SinkExt;

#[tokio::main]
async fn main() {
    let connect_addr = "wss://ws.okx.com:8443/ws/v5/public";

    let url = url::Url::parse(connect_addr).unwrap();

    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
    println!("WebSocket handshake has been successfully completed");

    let (mut write, mut read) = ws_stream.split();

    let mut ping_interval = tokio::time::interval(tokio::time::Duration::from_secs(5));

    loop {
        tokio::select! {
            msg = read.next() => {
                match msg {
                    Some(msg) => {
                        let msg = msg.unwrap();
                        println!("response: {}", msg);
                    },
                    None => break,
                }
            }
            _ = ping_interval.tick() => {
                let _ = write.send(Message::Text("ping".to_owned())).await;
            }
        }
    }
}
