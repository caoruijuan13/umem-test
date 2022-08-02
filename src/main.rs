use std::io;

mod socket;
mod socket_test;
mod test;
mod uio_test;
mod xsk_test;

#[tokio::main]
async fn main() -> io::Result<()> {
    // xsk_test::umem_test().await;
    // uio_test::test_pwritev();
    // uio_test::test_readv();
    test::umem_test().await;
    // socket_test::socket_test().await;
    Ok(())
}
