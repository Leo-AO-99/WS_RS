use futures_util::{future, pin_mut, StreamExt};
use tokio::io::AsyncReadExt;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

#[tokio::main]
async fn main() {
    // 修改连接地址为 wss://ws.okx.com:8443/ws/v5/public
    let connect_addr = "wss://ws.okx.com:8443/ws/v5/public";

    let url = url::Url::parse(connect_addr).unwrap();

    let (stdin_tx, stdin_rx) = futures_channel::mpsc::unbounded();
    tokio::spawn(read_stdin(stdin_tx));

    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
    println!("WebSocket handshake has been successfully completed");

    let (write, read) = ws_stream.split();

    let stdin_to_ws = stdin_rx.map(Ok).forward(write);
    let ws_to_stdout = {
        read.for_each(|message| async {
            let data = message.unwrap().into_data();
            println!("Received response: {:?}", String::from_utf8_lossy(&data));
        })
    };

    pin_mut!(stdin_to_ws, ws_to_stdout);
    future::select(stdin_to_ws, ws_to_stdout).await;
}

// Our helper method which will read data from stdin and send it along the
// sender provided.
async fn read_stdin(tx: futures_channel::mpsc::UnboundedSender<Message>) {
    let mut stdin = tokio::io::stdin();
    loop {
        let mut buf = vec![0; 1024];
        let n = match stdin.read(&mut buf).await {
            Err(_) | Ok(0) => break,
            Ok(n) => n,
        };
        buf.truncate(n);

        // 过滤掉回车符，然后发送消息
        if let Some(last) = buf.last() {
            if *last == b'\n' {
                let mut s = String::from_utf8(buf).unwrap();
                s.pop();
                if s.len() == 0 {
                    break;
                }
                tx.unbounded_send(Message::text(s)).unwrap();
                continue;
            }
        }
        tx.unbounded_send(Message::binary(buf)).unwrap();
    }
}
