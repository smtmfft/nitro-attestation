use aws_nitro_enclaves_nsm_api::api::{Request, Response};
use aws_nitro_enclaves_nsm_api::driver as nsm_driver;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
// use vsock::{VsockListener, VsockStream};
// use serde_bytes::ByteBuf;


#[derive(Serialize, Deserialize)]
struct SealedData {
    encrypted_data: Vec<u8>,
    required_pcr0: Vec<u8>,
}

pub fn get_pcr(pcr_index: u16) -> Vec<u8> {
    let request = Request::DescribePCR { index: pcr_index };

    let nsm_fd = nsm_driver::nsm_init();
    let response = nsm_driver::nsm_process_request(nsm_fd, request);
    nsm_driver::nsm_exit(nsm_fd);

    match response {
        Response::Attestation { document } => document,
        _ => panic!("nsm driver returned invalid response: {:?}", response),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_pcr0() {
        let pcr0 = get_pcr(0);
        println!("pcr0 = {:?}", pcr0);
    }
}
