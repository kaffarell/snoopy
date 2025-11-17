use std::os::fd::AsRawFd;

use anyhow::Context as _;
use aya::{
    maps::{MapData, perf::PerfEventArray},
    programs::{Xdp, XdpFlags},
    util::online_cpus,
};
use log::{debug, error, info};
use snoopy_common::ArpRequestInfo;
use tokio::{io::unix::AsyncFd, signal};

use crate::netlink::NetlinkManager;

pub async fn attach(
    source_interface: String,
    target_interface: String,
) -> Result<(), anyhow::Error> {
    // Bump the memlock rlimit. This is needed for older kernels that don't use the
    // new memcg based accounting, see https://lwn.net/Articles/837122/
    let rlim = libc::rlimit {
        rlim_cur: libc::RLIM_INFINITY,
        rlim_max: libc::RLIM_INFINITY,
    };
    let ret = unsafe { libc::setrlimit(libc::RLIMIT_MEMLOCK, &rlim) };
    if ret != 0 {
        debug!("remove limit on locked memory failed, ret is: {ret}");
    }

    // This will include your eBPF object file as raw bytes at compile-time and load it at
    // runtime. This approach is recommended for most real-world use cases. If you would
    // like to specify the eBPF program at runtime rather than at compile-time, you can
    // reach for `Bpf::load_file` instead.
    let mut ebpf = aya::Ebpf::load(aya::include_bytes_aligned!(concat!(
        env!("OUT_DIR"),
        "/snoopy"
    )))?;

    let program: &mut Xdp = ebpf.program_mut("snoopy").unwrap().try_into()?;
    program.load()?;
    program.attach(&source_interface, XdpFlags::default())
        .context("failed to attach the XDP program with default flags - try changing XdpFlags::default() to XdpFlags::SKB_MODE")?;

    let mut perf_array = PerfEventArray::try_from(
        ebpf.take_map("EVENTS")
            .ok_or(anyhow::anyhow!("unable to get events map"))?,
    )?;

    run(&mut perf_array, target_interface).await?;

    Ok(())
}

async fn run(
    event_array: &mut PerfEventArray<MapData>,
    target_interface: String,
) -> Result<(), anyhow::Error> {
    for cpu_id in online_cpus().map_err(|(_, error)| error)? {
        let mut buf = event_array.open(cpu_id, None)?;

        let target_interface = target_interface.clone();
        tokio::spawn(async move {
            let async_fd = AsyncFd::new(buf.as_raw_fd()).unwrap();

            let mut buffers = (0..10)
                .map(|_| bytes::BytesMut::with_capacity(1024))
                .collect::<Vec<_>>();

            loop {
                let mut guard = async_fd.readable().await.unwrap();

                let events = match buf.read_events(&mut buffers) {
                    Ok(events) => events,
                    Err(e) => {
                        error!("Error: {}", e);
                        continue;
                    }
                };

                for buf in buffers.iter_mut().take(events.read) {
                    let ptr = buf.as_ptr() as *const ArpRequestInfo;
                    let data = unsafe { ptr.read_unaligned() };
                    debug!("ARP request intercepted: {:?}", data);

                    match check_and_insert_neighbor(data, target_interface.clone()).await {
                        Ok(()) => {}
                        Err(err) => {
                            error!("error when inserting neighbor into netlink: {err:#}");
                            continue;
                        }
                    }
                }
                guard.clear_ready();
            }
        });
    }

    let ctrl_c = signal::ctrl_c();
    println!("Waiting for Ctrl-C...");
    ctrl_c.await?;
    println!("Exiting...");

    Ok(())
}

async fn check_and_insert_neighbor(
    data: ArpRequestInfo,
    target_interface: String,
) -> Result<(), anyhow::Error> {
    let netlink_manager = NetlinkManager::new().context("error connecting to netlink")?;

    netlink_manager
        .add_neighbor(&target_interface, data.src_ip.into(), data.src_mac)
        .await
        .context("error inserting neighbor")?;
    info!(
        "inserted neighbor: {} - {} on dev {}",
        data.src_ip.map(|s| format!("{s}")).join("."),
        data.src_mac.map(|s| format!("{:x}", s)).join(":"),
        target_interface
    );

    Ok(())
}
