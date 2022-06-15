use xsk_rs::{config::{FrameSize, UmemConfig}, umem::Umem};

use nix::sys::uio::*;
use nix::unistd::*;
#[cfg(not(target_os = "redox"))]
use std::io::IoSliceMut;

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

    // socket
    let mut iovecs = Vec::with_capacity(descs.len());
    
    // get ownership
    let write_num = 1;
    for i in 0..write_num {
        let index = q.pop_front().unwrap() as usize;
        // write and assert
        unsafe {
            let (mut h, mut d) = umem.frame_mut(&mut descs[index]);

            iovecs.push(IoSliceMut::new(d.contents_mut()));

            // h.cursor().write_all(format!("hello-{}", index).as_bytes()).unwrap();
            // d.cursor().write_all(format!("world-{}", index).as_bytes()).unwrap();

            // println!("descs: {:?}", descs);
            // println!("headroom-{}: {:?}", index, str::from_utf8(umem.headroom(&descs[index]).contents()));
            // println!("data-{}: {:?}", index, str::from_utf8(umem.data(&descs[index]).contents()));

            // assert_eq!(umem.headroom(&descs[index]).contents(), b"hello");
            // assert_eq!(umem.headroom_mut(&mut descs[index]).contents(), b"hello");

            // assert_eq!(umem.data(&descs[index]).contents(), b"world");
            // assert_eq!(umem.data_mut(&mut descs[index]).contents(), b"world");
        }
    }

    let to_write = super::uio_test::gen_data();
    super::uio_test::check_read(&mut iovecs, to_write);

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
