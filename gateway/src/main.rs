mod authority;
mod http;
mod options;

use anyhow::Result;
use authority::HttpAuthority;
use clap::Parser;
use log::*;
use options::Options;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::net::UdpSocket;
use tokio::task;
use trust_dns_server::authority::{Authority, Catalog};
use trust_dns_server::server::ServerFuture;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let options = Options::parse();
    info!("options: {:?}", options);

    let authority = HttpAuthority::new(options.origin, options.endpoint)?;

    let catalog = {
        let mut catalog = Catalog::new();
        catalog.upsert(authority.origin().clone(), Box::new(Arc::new(authority)));
        catalog
    };

    let server = {
        let mut server = ServerFuture::new(catalog);
        for udp_socket in &options.socket_addrs {
            let udp_socket = UdpSocket::bind(udp_socket).await?;
            server.register_socket(udp_socket);
        }
        for tcp_listener in &options.socket_addrs {
            let tcp_listener = TcpListener::bind(tcp_listener).await?;
            let timeout = Duration::from_secs(10);
            server.register_listener(tcp_listener, timeout);
        }
        task::spawn(server.block_until_done())
    };

    info!("Server ready");
    server.await??;

    info!("Exiting...");
    Ok(())
}
