use std::{io};

mod xsk_test;
mod uio_test;
mod socket_test;

#[tokio::main]
async fn main() -> io::Result<()> {
    xsk_test::umem_test().await;
    // uio_test::test_pwritev();
    // uio_test::test_readv();

    Ok(())
}
