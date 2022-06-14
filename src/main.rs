use std::vec;
use std::{convert::TryInto, io::Write, str};
use std::collections::VecDeque;
use xsk_rs::{config::{FrameSize, UmemConfig}, umem::Umem};
use tokio::io::{self, AsyncReadExt};


async fn umem_test() {
    // config
    let frame_size = 2048;
    let frame_count = 3;
    let frame_headroom = 32;

    // init
    let (umem, mut descs) = Umem::new(
        UmemConfig::builder()
        .frame_headroom(frame_headroom)
        .frame_size(FrameSize::new(frame_size).unwrap()).build().unwrap(),
        frame_count.try_into().unwrap(),
        false,
    )
    .unwrap();

    // queue
    // let q: VecDeque<u32> = VecDeque::with_capacity(10);
    let mut q: VecDeque<u32> = VecDeque::new();
    for i in 0..frame_count {
        q.push_back(i);
    }

    // get ownership
    let write_num = 3;
    for i in 0..write_num {
        let index = q.pop_front().unwrap() as usize;
        // write and assert
        unsafe {
            let (mut h, mut d) = umem.frame_mut(&mut descs[index]);

            //let index_bytes = (1000+index).to_le_bytes();
            //println!("index = {}, bytes = {:?}", index, index_bytes);
            
            h.cursor().write_all(format!("hello-{}", index).as_bytes()).unwrap();
            //h.cursor().write_all(&index_bytes).unwrap();
            d.cursor().write_all(format!("world-{}", index).as_bytes()).unwrap();
            //d.cursor().write_all(&index_bytes).unwrap();

            println!("descs: {:?}", descs);
            println!("headroom-{}: {:?}", index, str::from_utf8(umem.headroom(&descs[index]).contents()));
            println!("data-{}: {:?}", index, str::from_utf8(umem.data(&descs[index]).contents()));

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
