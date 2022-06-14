use std::vec;
use std::{convert::TryInto, io::Write, str};
use std::collections::VecDeque;
use xsk_rs::{config::UmemConfig, umem::Umem};
use tokio::io::{self, AsyncReadExt};


async fn umem_test() {
    // config
    let frame_size = 2048;
    let frame_count = 64;
    let frame_headroom = 32;

    // init
    let (umem, mut descs) = Umem::new(
        UmemConfig::builder()
        .frame_headroom(frame_headroom)
        .frame_size(frame_size).build().unwrap(),
        frame_count.try_into().unwrap(),
        false,
    )
    .unwrap();

    // queue
    // let q: VecDeque<u32> = VecDeque::with_capacity(10);
    let q: VecDeque<u32> = VecDeque::new();
    for i in 0..frame_count {
        q.push_back(i);
    }

    // get ownership
    let write_num = 3;
    for i in 0..write_num {
        let index = q.pop_back();
        // write and assert
        unsafe {
            let (mut h, mut d) = umem.frame_mut(&mut descs[index]);

            h.cursor().write_all(b"hello").unwrap();
            h.cursor().write_all(index).unwrap();
            d.cursor().write_all(b"world").unwrap();
            d.cursor().write_all(index).unwrap();

            println!("descs: {:?}", descs);
            println!("headroom-{}: {:?}", index, umem.headroom(&descs[index]).contents());
            println!("data-{}: {:?}", index, umem.data(&descs[index]).contents());

            // assert_eq!(umem.headroom(&descs[index]).contents(), b"hello");
            // assert_eq!(umem.headroom_mut(&mut descs[index]).contents(), b"hello");

            // assert_eq!(umem.data(&descs[index]).contents(), b"world");
            // assert_eq!(umem.data_mut(&mut descs[index]).contents(), b"world");
        }
    }

    // read
    let rx_q = vec![0, 2];
    for index in rx_q {
        unsafe {
            println!("descs: {:?}", descs);
            println!("headroom-{}: {:?}", index, umem.headroom(&descs[index]).contents());
            println!("data-{}: {:?}", index, umem.data(&descs[index]).contents());
        }
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    umem_test().await;

    Ok(())
}