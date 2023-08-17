mod aggregator;
mod exchanges;
mod service;

use aggregator::Aggregator;
use clap::{arg, Parser};
use exchanges::{Binance, Bitstamp, Orderbook};
use grpc::keyrock_server::KeyrockServer;
use service::KeyrockService;
use tokio::sync::{mpsc::channel, OnceCell};
use tonic::transport::Server;

#[derive(Parser, Clone)]
struct AppCfg {
    /// Set a financial instrument to stream
    #[arg(short = 'i', long, default_value = "ethbtc", value_name = "ethbtc")]
    instr: String,

    /// Display debug information when true
    #[arg(short = 'd', long)]
    dbg: bool,
}

#[tokio::main]
async fn main() {
    // Parse any command-line configuration.
    let app_cfg = AppCfg::parse();

    // Set the application-wide debug setting.
    if let Err(e) = DBG.set(app_cfg.dbg) {
        println!("err: {:?}", e);
    }

    // Communicate between receiver threads and
    // aggregator thread with a channel.
    let (tx_agg, rx_agg) = channel::<Orderbook>(64);
    let (tx_grpc, rx_grpc) = channel::<Orderbook>(64);

    let keyrock_service = KeyrockService::new(rx_grpc);

    // Start the Binance input stream receiver.
    let instr = app_cfg.instr.clone();
    let tx_agg_clone = tx_agg.clone();
    tokio::spawn(async move {
        let binance = Binance::new(instr);
        binance.stream_orderbook(tx_agg_clone).await;
    });

    // Start the Bitstamp input stream receiver.
    let instr = app_cfg.instr.clone();
    let tx_agg_clone = tx_agg.clone();
    tokio::spawn(async move {
        let bitstamp = Bitstamp::new(instr);
        bitstamp.stream_orderbook(tx_agg_clone).await;
    });

    // Start the order book aggregator.
    tokio::spawn(async move {
        let mut aggregator = Aggregator::new();
        aggregator.aggregate_streams(rx_agg, tx_grpc).await;
    });

    // Start the gRPC server for the output stream.
    let addr = "[::1]:10000".parse().unwrap();
    let svc = KeyrockServer::new(keyrock_service);
    Server::builder()
        .add_service(svc)
        .serve(addr)
        .await
        .unwrap();
}

/// `10` bids or asks.
pub const BID_ASK_LEN: usize = 10;

/// Application-wide configuration to display debug information.
pub static DBG: OnceCell<bool> = OnceCell::const_new();
