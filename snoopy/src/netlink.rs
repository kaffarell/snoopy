use std::net::IpAddr;

use futures::stream::TryStreamExt;
use log::debug;
use rtnetlink::{Handle, new_connection, packet_route::neighbour::NeighbourState};

pub async fn netlink_add_neighbor(
    interface: &str,
    ip_addr: IpAddr,
    ll_addr: [u8; 6],
) -> Result<(), anyhow::Error> {
    let (connection, handle, _) = new_connection()?;
    tokio::spawn(connection);

    add_neighbor(interface, ip_addr, ll_addr, handle.clone()).await
}

async fn add_neighbor(
    link_name: &str,
    ip_addr: IpAddr,
    ll_addr: [u8; 6],
    handle: Handle,
) -> Result<(), anyhow::Error> {
    let mut links = handle
        .link()
        .get()
        .match_name(link_name.to_string())
        .execute();
    if let Some(link) = links.try_next().await? {
        handle
            .neighbours()
            .add(link.header.index, ip_addr)
            .replace()
            .link_local_address(&ll_addr)
            .state(NeighbourState::Reachable)
            .execute()
            .await?;
        debug!("Inserted neighbor!");
    }

    Ok(())
}
