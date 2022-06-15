use std::{io};

mod xsk_test;
mod uio_test;
mod socket_test;

#[tokio::main]
async fn main() -> io::Result<()> {
    // umem_test().await;
    // xsk_test();
    // test_pwritev();
    uio_test::test_readv();

    Ok(())
}
