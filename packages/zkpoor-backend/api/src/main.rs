use zkpoor_api::server::run_server;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .unwrap_or(8080);

    run_server(port).await
}