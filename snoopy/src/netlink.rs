use std::net::IpAddr;

use futures::stream::TryStreamExt;
use log::debug;
use rtnetlink::{Handle, new_connection, packet_route::neighbour::NeighbourState};
use tokio::task::JoinHandle;

pub struct NetlinkManager {
    handle: Handle,
    _connection: JoinHandle<()>,
}

impl NetlinkManager {
    pub fn new() -> Result<NetlinkManager, anyhow::Error> {
        let (connection, handle, _) = new_connection()?;
        let connection = tokio::spawn(connection);

        Ok(NetlinkManager {
            handle,
            _connection: connection,
        })
    }

    /// Add neighbor with ip address and mac address to the specific interface
    pub async fn add_neighbor(
        &self,
        link_name: &str,
        ip_addr: IpAddr,
        ll_addr: [u8; 6],
    ) -> Result<(), anyhow::Error> {
        let mut links = self
            .handle
            .link()
            .get()
            .match_name(link_name.to_string())
            .execute();
        if let Some(link) = links.try_next().await? {
            self.handle
                .neighbours()
                .add(link.header.index, ip_addr)
                .replace()
                .link_local_address(&ll_addr)
                .state(NeighbourState::Stale)
                .execute()
                .await?;
            debug!("Inserted neighbor!");
        }

        Ok(())
    }
}
