use futures_util::stream::StreamExt;
use futures_util::SinkExt;
use http::Uri;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio_websockets::{ClientBuilder, Message};

#[tokio::main]
async fn main() -> Result<(), tokio_websockets::Error> {
    let (mut ws_stream, _) =
        ClientBuilder::from_uri(Uri::from_static("ws://ec2-3-21-236-80.us-east-2.compute.amazonaws.com:2000"))
            .connect()
            .await?;

    let stdin = tokio::io::stdin();
    let mut stdin = BufReader::new(stdin).lines();

    // Prompt for username
    println!("Enter your username:");
    let mut username = String::new();
    if let Ok(line) = stdin.next_line().await {
        username = line.unwrap_or("Guest".to_string()); // Use unwrap_or instead of unwrap_or_else
    }

    // Send username to the server
    ws_stream.send(Message::text(username.clone())).await?;

    loop {
        tokio::select! {
            incoming = ws_stream.next() => {
                match incoming {
                    Some(Ok(msg)) => {
                        if let Some(text) = msg.as_text() {
                            println!("From server: {}", text);
                        }
                    }
                    Some(Err(err)) => return Err(err.into()),
                    None => return Ok(()),
                }
            }
            res = stdin.next_line() => {
                match res {
                    Ok(None) => return Ok(()),
                    Ok(Some(line)) => ws_stream.send(Message::text(line)).await?,
                    Err(err) => return Err(err.into()),
                }
            }
        }
    }
}