mod zip;

#[tokio::main]
async fn main() -> Result<(), String> {
    zip::fetch().await
}
