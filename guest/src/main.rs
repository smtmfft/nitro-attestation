use std::{thread, time::Duration};

mod attest;

use attest::get_pcr;
use attest::vsock_client::VsockClient;
use attest::vsock_log::init_logging;

fn main() {
    let client = VsockClient::new();

    // 初始化日志系统
    init_logging(client).expect("init_logging");

    log::info!("Hello, world!");

    let pcr0 = get_pcr(0);
    log::info!("pcr0 = {:?}", pcr0);

    // some time delay
    for i in 0..5 {
        log::info!("Still running... {} seconds passed", i * 10);
        // let pcri = get_pcr(i);
        // println!("pcr{:?} = {:?}", i, pcri);
        thread::sleep(Duration::from_secs(5));
    }
}
