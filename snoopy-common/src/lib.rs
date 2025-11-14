#![no_std]

#[derive(Debug)]
#[repr(C)]
pub struct ArpRequestInfo {
    pub src_ip: [u8; 4],
    pub src_mac: [u8; 6],
}
