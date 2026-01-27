mod chain {
    pub mod biophysical_runtime;
    pub mod host_node;
}

use chain::biophysical_runtime::aln_did::ALNDID;
use chain::host_node::bootstrap_single_host_node;
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let host_id = ALNDID {
        id: "bostrom18sd2ujv24ual9c9pshtxys6j8knh6xaead9ye7".to_string(),
        shard: "phx-main".to_string(),
    };

    let bind: SocketAddr = "127.0.0.1:7070".parse().unwrap();
    let (_handle, mut gossip_rx) = bootstrap_single_host_node(host_id, bind).await?;

    tokio::spawn(async move {
        while let Some(frame) = gossip_rx.recv().await {
            println!("[GOSSIP] {:?}", frame);
        }
    });

    // Node runs indefinitely.
    futures::future::pending::<()>().await;
    Ok(())
}
