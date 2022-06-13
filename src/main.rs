use xsk_rs::umem;

use tokio::fs::File;
use tokio::io::{self, AsyncReadExt};

async fn test() {
    let frame_count = 64;

    let (umem, descs) = Umem::new(
        UmemConfig::default(),
        frame_count.try_into().unwrap(),
        false,
    )
    .unwrap();

}

async fn frame_test() {
    let (umem, mut descs) = Umem::new(
        UmemConfig::builder().frame_headroom(32).build().unwrap(),
        64.try_into().unwrap(),
        false,
    )
    .unwrap();

    unsafe {
        let (mut h, mut d) = umem.frame_mut(&mut descs[0]);

        h.cursor().write_all(b"hello").unwrap();
        d.cursor().write_all(b"world").unwrap();

        println!("descs: {:?}", descs);

        assert_eq!(umem.headroom(&descs[0]).contents(), b"hello");
        assert_eq!(umem.headroom_mut(&mut descs[0]).contents(), b"hello");

        assert_eq!(umem.data(&descs[0]).contents(), b"world");
        assert_eq!(umem.data_mut(&mut descs[0]).contents(), b"world");
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    // let mut f = File::open("test.txt").await?;
    // let mut buffer = [0; 10];

    // // read up to 10 bytes
    // let n = f.read(&mut buffer[..]).await?;

    // println!("The bytes: {:?}", &buffer[..n]);

    frame_test().await;

    Ok(())
}