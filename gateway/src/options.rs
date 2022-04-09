use clap::Clap;
use std::net::SocketAddr;

#[derive(Debug, Clap)]
#[clap()]
pub struct Options {
    /// Socket addresses to listen on.
    #[clap(short = 'l', long, env, default_value = "0.0.0.0:53")]
    pub socket_addrs: Vec<SocketAddr>,

    /// Origin of this DNS server.
    #[clap(long, env, default_value = ".")]
    pub origin: String,

    /// HTTP endpoint to adapt to.
    #[clap(long, env)]
    pub endpoint: String,
}
