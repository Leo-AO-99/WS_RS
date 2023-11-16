use csv::WriterBuilder;
use futures_util::StreamExt;
use tokio_tungstenite::{connect_async, tungstenite::{protocol::Message, Result}};
use futures_util::SinkExt;
use serde_json::{Value, json};
use std::{time::SystemTime, sync::{Arc, Mutex}};

pub fn get_ts_s() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

pub fn get_ts_ms() -> u128 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis()
}

#[tokio::main]
async fn main() -> Result<()>{
    let connect_addr = "wss://wsaws.okx.com:8443/ws/v5/public";

    let url = url::Url::parse(connect_addr).unwrap();

    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
    println!("WebSocket handshake has been successfully completed");

    let (mut write, mut read) = ws_stream.split();

    let listing_sub: Value = json!({
        "op": "subscribe",
        "args": [{
            "channel": "bbo-tbt",
            "instId": "BTC-USDT"
        }]
    });

    let listing_unsub = serde_json::json!({
        "op": "unsubscribe",
        "args": [{
            "channel": "bbo-tbt",
            "instId": "BTC-USDT"
        }]
    });

    

    println!("{} send subscribe", get_ts_ms());
    write.send(Message::Text(listing_sub.to_string())).await?;

    let done = Arc::new(Mutex::new(false));
    let data_vec = Arc::new(Mutex::new(Vec::new()));
    let vec_in = Arc::clone(&data_vec);
    let vec_out = Arc::clone(&data_vec);

    let mut ping_interval = tokio::time::interval(tokio::time::Duration::from_secs(20));
    let dur = 5;
    let mut record_interval = tokio::time::interval(tokio::time::Duration::from_secs(dur));


    tokio::spawn({
        let done = Arc::clone(&done);
        async move {
            tokio::signal::ctrl_c().await.unwrap();
            println!("ctrl_c received");
            *done.lock().unwrap() = true;
        }
    });


    loop {
        tokio::select! {
            msg = read.next() => {
                match msg {
                    Some(msg) => {
                        if *done.lock().unwrap() {
                            write.send(Message::Text(listing_unsub.to_string())).await?;
                            break;
                        }
                        let msg = msg?;
                        if msg.to_string().eq("pong") {
                            continue;
                        }
                        let resp: Value = serde_json::from_str(&msg.to_string()).unwrap();
                        
                        if let Some(data) = resp.get("data") {
                            let data = data.as_array().unwrap().get(0).unwrap();
                            let server_ts: u128 = data.get("ts").unwrap().as_str().unwrap().parse().unwrap();


                            vec_in.lock().unwrap().push((get_ts_ms(), server_ts));
                        }
                    },
                    None => {
                        write.send(Message::Text(listing_unsub.to_string())).await?;
                        println!("done {}", get_ts_s());
                        break;
                    },
                }
            }
            _ = ping_interval.tick() => {
                write.send(Message::Text("ping".to_owned())).await?;
            }
            _ = record_interval.tick() => {
                let path = format!("{}/csv/{}.csv", env!("CARGO_MANIFEST_DIR"), get_ts_s());
                let mut writer = WriterBuilder::new().from_path(path).unwrap();
                
                let mut v = vec_out.lock().unwrap();
                v.resize(100, (0, 0));

                for (value1, value2) in v.iter() {
                    writer.write_record(&[value1.to_string(), value2.to_string()]).unwrap();
                }
            
                // 完成写入并保存文件
                writer.flush()?;

                v.clear();
            }
        }
    }

    Ok(())
}
