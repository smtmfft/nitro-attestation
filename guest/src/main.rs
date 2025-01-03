use std::{thread, time::Duration};

mod attest;

use attest::get_pcr;
use attest::vsock_client::VsockClient;
use attest::vsock_log::init_logging;

use anyhow::Error;
use attest::client::Client;

fn main() -> Result<(), Error> {
    // 配置 vsock 端口
    const CMD_PORT: u32 = 5000;
    const LOG_PORT: u32 = 5001;

    println!("Setup client...");

    // 创建并运行 client
    let client = Client::new(CMD_PORT, LOG_PORT)?;
    println!("Starting client...");
    client.run()?;

    // 因为 run() 会在新线程中处理请求，所以需要防止主线程退出
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    Ok(())
}

fn deprecated_main() {
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
