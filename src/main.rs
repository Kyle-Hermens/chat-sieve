mod youtube;

#[tokio::main]
async fn main() {
    youtube::fetch_youtube_live_chat().await;
}
