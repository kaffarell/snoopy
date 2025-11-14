#![no_std]
#![no_main]

use core::mem;

use aya_ebpf::{
    bindings::xdp_action,
    macros::{map, xdp},
    maps::PerfEventArray,
    programs::XdpContext,
};
use network_types::{
    arp::ArpHdr,
    eth::{EthHdr, EtherType},
};
use snoopy_common::ArpRequestInfo;

#[map]
static EVENTS: PerfEventArray<ArpRequestInfo> = PerfEventArray::new(0);

#[xdp]
pub fn snoopy(ctx: XdpContext) -> u32 {
    match try_snoopy(ctx) {
        Ok(ret) => ret,
        Err(_) => xdp_action::XDP_PASS,
    }
}

fn ptr_at<T>(ctx: &XdpContext, offset: usize) -> Result<*const T, ()> {
    let start = ctx.data();
    let end = ctx.data_end();
    let len = mem::size_of::<T>();

    if start + offset + len > end {
        return Err(());
    }

    Ok((start + offset) as *const T)
}

fn try_snoopy(ctx: XdpContext) -> Result<u32, ()> {
    let ethhdr: *const EthHdr = ptr_at(&ctx, 0)?;
    // match on arp
    match unsafe { (*ethhdr).ether_type() } {
        Ok(EtherType::Arp) => {}
        _ => return Ok(xdp_action::XDP_PASS),
    }

    let arphdr: *const ArpHdr = ptr_at(&ctx, EthHdr::LEN)?;
    let arp_oper = unsafe { (&*arphdr) as &ArpHdr }.oper();

    // filter on arp requests (arp replies have this bit set to 2)
    if arp_oper == 1 {
        let info = ArpRequestInfo {
            src_ip: unsafe { (&*arphdr) as &ArpHdr }.spa(),
            src_mac: unsafe { (&*arphdr) as &ArpHdr }.sha(),
        };
        EVENTS.output(&ctx, &info, 0);
    }

    Ok(xdp_action::XDP_PASS)
}

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[unsafe(link_section = "license")]
#[unsafe(no_mangle)]
static LICENSE: [u8; 13] = *b"Dual MIT/GPL\0";
