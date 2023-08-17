use grpc::{keyrock_client::KeyrockClient, Empty};
use tonic::Request;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = KeyrockClient::connect("http://[::1]:10000").await?;

    let mut stream = client.summary(Request::new(Empty {})).await?.into_inner();

    while let Some(orderbook) = stream.message().await? {
        println!("{:?}", orderbook);
    }
    Ok(())
}
