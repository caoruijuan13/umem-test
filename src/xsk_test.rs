use nix::sys::uio::*;
use nix::unistd::*;
use std::collections::VecDeque;
#[cfg(not(target_os = "redox"))]
use std::io::IoSliceMut;
use std::{convert::TryInto, io::Write, slice, str};
use xsk_rs::{
    config::{FrameSize, UmemConfig},
    umem::Umem,
};

pub async fn umem_test() {
    // config
    let frame_size = 2048;
    let frame_count = 3;
    let frame_headroom = 256;

    let config = UmemConfig::builder()
        .frame_headroom(frame_headroom)
        .frame_size(FrameSize::new(frame_size).unwrap())
        .build()
        .unwrap();
    println!(
        "config={:?} -{}-{}",
        config,
        config.xdp_headroom(),
        config.mtu()
    );

    // init
    let (umem, mut descs) = Umem::new(config, frame_count.try_into().unwrap(), false).unwrap();
    let mut desc0 = descs[0];
    let mut desc1 = descs[1];

    println!("umem = {:?}", umem);
    println!("umem region = {:?}", umem.mem);
    println!("descs = {:?}", descs);

    // queue
    // let q: VecDeque<u32> = VecDeque::with_capacity(10);
    let mut q: VecDeque<u32> = VecDeque::new();
    for i in 0..frame_count {
        let ptr = unsafe { umem.mem.data_ptr(&descs[i as usize]) };
        println!("{}'s data_ptr = {:?}", i, ptr);
        q.push_back(i);
    }

    // socket
    let mut iovecs = Vec::with_capacity(descs.len());

    // get ownership
    let write_num = 3;

    for i in 0..write_num {
        let index = q.pop_front().unwrap() as usize;
        // let headroom_ptr = unsafe { umem.headroom_ptr(descs[index]) };
        // let data_ptr = unsafe { umem.data_ptr(descs[index]) };

        // let headroom =
        //     unsafe { slice::from_raw_parts_mut(headroom_ptr, umem.layout.frame_headroom) };

        // let data = unsafe { slice::from_raw_parts_mut(data_ptr, umem.layout.mtu) };

        // let (h, d) = (
        //     HeadroomMut::new(&mut desc.lengths.headroom, headroom),
        //     DataMut::new(&mut desc.lengths.data, data),
        // )

        // write and assert
        // let (mut h, mut d) = unsafe {
        // println!("desc0={:?}\n desc[index]={:?}", desc0, descs[index]);
        // umem.frame_mut(&mut desc0)
        // umem.frame_mut(&mut descs[index])
        //iovecs.push(IoSliceMut::new(d.contents_mut()));

        // h.cursor().write_all(format!("hello-{}", index).as_bytes()).unwrap();
        // d.cursor().write_all(format!("world-{}", index).as_bytes()).unwrap();

        // println!("descs: {:?}", descs);
        // println!("headroom-{}: {:?}", index, str::from_utf8(umem.headroom(&descs[index]).contents()));
        // println!("data-{}: {:?}", index, str::from_utf8(umem.data(&descs[index]).contents()));

        // assert_eq!(umem.headroom(&descs[index]).contents(), b"hello");
        // assert_eq!(umem.headroom_mut(&mut descs[index]).contents(), b"hello");

        // assert_eq!(umem.data(&descs[index]).contents(), b"world");
        // assert_eq!(umem.data_mut(&mut descs[index]).contents(), b"world");
        // };
        // println!("---{:?}\n---{:?}\n---{:?}", d.cursor().zero_out(), d.cursor().pos(), d.cursor().buf_len());
        // h.cursor().write_all(format!("hello-{}", index).as_bytes()).unwrap();
        // d.cursor().write_all(format!("world-{}", index).as_bytes()).unwrap();
        //println!("---{:?}\n---{:?}\n---{:?}", d, d.cursor().buf(), index);
        // let mut buf = slice::from_raw_parts(umem.mem.data_ptr(&mut descs[index]), 128);
        // iovecs.push(IoSliceMut::new(&mut buf));

        // let data_ptr = unsafe { umem.mem.data_ptr(&mut descs[index]) };
        // let d = unsafe { slice::from_raw_parts_mut(data_ptr, super::uio_test::WRITE_LEN/2) };

        let data_ptr = unsafe { umem.mem.data_ptr(&descs[index]) };

        let d = unsafe { slice::from_raw_parts_mut(data_ptr, super::uio_test::WRITE_LEN / 3) };
        descs[index].adjust_data(super::uio_test::WRITE_LEN / 3);

        // iovecs.push(IoSliceMut::new(DataMut::new(super::uio_test::WRITE_LEN/2, data).buf_mut(super::uio_test::WRITE_LEN/2)));

        iovecs.push(IoSliceMut::new(d));

        // let mut d = unsafe{ umem.data_mut(&mut descs[index]) };
        // iovecs.push(IoSliceMut::new(d.buf_mut(super::uio_test::WRITE_LEN/2)));
        // iovecs.push(IoSliceMut::new(d.contents_mut()));
        // iovecs.push(IoSliceMut::new(d.cursor().buf()));
    }

    // let index = q.pop_front().unwrap() as usize;
    // let (mut h, mut d) = unsafe {
    //     // umem.frame_mut(&mut desc1)
    //     umem.frame_mut(&mut descs[index])
    // };
    // // let mut d = unsafe{ umem.data_mut(&mut descs[index]) };

    // iovecs.push(IoSliceMut::new(d.buf_mut(super::uio_test::WRITE_LEN/2)));

    println!("iovecs-{}:{:?}", iovecs.len(), iovecs);

    // println!("umem = {:?}", umem);
    // println!("umem region = {:?}", umem.mem);
    // println!("descs = {:?}", descs);

    super::uio_test::check_read(&super::uio_test::gen_data(), &mut iovecs);

    // read
    let rx_q = vec![0, 1, 2];
    for index in rx_q {
        unsafe {
            println!("descs: {:?}", descs);
            println!(
                "headroom-{}: {:?}",
                index,
                umem.headroom(&descs[index]).contents()
            );
            println!("data-{}: {:?}", index, umem.data(&descs[index]).contents());
        }
    }
}
