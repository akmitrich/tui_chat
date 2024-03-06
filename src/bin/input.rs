use redis::AsyncCommands;

#[tokio::main]
async fn main() {
    let client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let mut con = client.get_tokio_connection().await.unwrap();
    let opts = redis::streams::StreamReadOptions::default()
        .count(10)
        .block(10000);
    let result: Result<redis::streams::StreamReadReply, _> =
        con.xread_options(&["42"], &["$"], &opts).await;
    println!("{:?}", result);
}
