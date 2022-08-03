use nix::sys::uio::*;
use nix::unistd::*;
use std::collections::VecDeque;
#[cfg(not(target_os = "redox"))]
use std::io::{IoSlice, IoSliceMut};
use std::sync::{Arc, Mutex, RwLock};
use std::{convert::TryInto, io::Write, slice, str};
use xsk_rs::{
    config::{FrameSize, UmemConfig},
    umem::frame::FrameDesc,
    umem::Umem,
    umem::Queue,
};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref UMEMINFO: RwLock<UmemInfo> = RwLock::new(UmemInfo::default());
}

pub struct UmemInfo {
    pub umem: Option<Umem>,
    pub descs: Vec<FrameDesc>,
    pub queue: Queue,
}

impl Default for UmemInfo {
    fn default() -> Self {
        Self {
            umem: None,
            descs: Vec::new(),
            queue: Queue::default(),
        }
    }
}

// init
pub fn init_umem(frame_size: u32, frame_count: u32, frame_headroom: u32) -> (Umem, Vec<FrameDesc>, Queue) {
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
    let (umem, mut descs, queue) = Umem::new(config, frame_count.try_into().unwrap(), false).unwrap();
    println!("umem = {:?}", umem);
    println!("umem region = {:?}", umem.mem);
    println!("descs = {:?}", descs);

    (umem, descs, queue)
}

pub async fn umem_test() {
    // config
    let frame_size = 2048;
    let frame_count = 3;
    let frame_headroom = 256;

    let (umem, mut descs, queue) = init_umem(frame_size, frame_count, frame_headroom);
    // UMEMINFO.write().unwrap().umem = Some(umem);
    // UMEMINFO.write().unwrap().queue.push_back(1);
    // UMEMINFO.write().unwrap().queue.push_back(0);
    UMEMINFO.write().unwrap().umem = Some(umem);
    UMEMINFO.write().unwrap().descs = descs;
    UMEMINFO.write().unwrap().queue = queue;

    // let umem = Mutex::new(umem);
    // let descs = Mutex::new(descs);
    let h = std::thread::spawn( move || {
        println!("------h1");
        let umem = UMEMINFO.write().unwrap().umem.clone().unwrap();
        println!("------h1-1");
        let mut descs = UMEMINFO.write().unwrap().descs.clone();
        let mut queue = UMEMINFO.write().unwrap().queue.clone();
        println!("------h1-2");
        test(&umem, &mut descs, &mut queue);
        println!("------h1-3");
        // test();
    });

    println!("------h2");
    let umem = UMEMINFO.write().unwrap().umem.clone().unwrap();
    let mut descs = UMEMINFO.write().unwrap().descs.clone();
    let mut queue = UMEMINFO.write().unwrap().queue.clone();

    // let mut descs = Vec::new();
    // let queue_id = UMEMINFO.write().unwrap().queue.pop_front();
    // println!("queue id = {:?}", queue_id);
    test(&umem, &mut descs, &mut queue);
    // test(&umem, &mut descs).await;
    // test();
    // test(&umem, &mut descs);
    h.join().unwrap();
}

/*
// pub fn test(umem: &Umem, descs: &mut Vec<FrameDesc>) {
pub fn test_static() {
    // let umem = UMEMINFO.read().unwrap().umem.as_ref().unwrap();
    // let mut descs = UMEMINFO.write().unwrap().descs.as_ref();

    // socket
    let mut iovecs = Vec::with_capacity(UMEMINFO.write().unwrap().descs.len());

    // get frame
    let write_num = 3;
    for i in 0..write_num {
        // let mut desc = umem.get_frame().unwrap();
        let length = super::uio_test::WRITE_LEN / 3;
        let slice = unsafe { UMEMINFO.read().unwrap().umem.unwrap().mem.get_data_IoSliceMut(&mut UMEMINFO.write().unwrap().descs[i], length) };
        if i == 0 {
            let buf = "hello world".as_bytes();
            println!("buf={:?}", buf);
            let head_slice = unsafe { UMEMINFO.read().unwrap().umem.unwrap().mem.get_headroom_IoSliceMut(&mut UMEMINFO.write().unwrap().descs[i], buf.len()) };
            super::uio_test::check_read(&buf, &mut vec![head_slice]);
        }
        // descs[index].adjust_data(length);
        iovecs.push(slice);
    }

    println!("iovecs-{}:{:?}", iovecs.len(), iovecs);

    // println!("umem = {:?}", umem);
    // println!("umem region = {:?}", umem.mem);
    // println!("descs = {:?}", descs);

    super::uio_test::check_read(&super::uio_test::gen_data(), &mut iovecs);

    // read
    // let mut iovecs = Vec::with_capacity(descs.len());
    // let rx_q = vec![0, 1, 2];
    // for index in rx_q {
    //     unsafe {
    //         println!("descs: {:?}", descs);
    //         println!(
    //             "headroom-{}: {:?}",
    //             index,
    //             umem.headroom(&descs[index]).contents()
    //         );
    //         println!(
    //             "data-{}:{}- {:?}",
    //             index,
    //             umem.data(&descs[index]).contents().len(),
    //             umem.data(&descs[index]).contents()
    //         );
    //     }
    //     iovecs.push(unsafe { umem.mem.get_data_IoSlice(&descs[index]) });
    // }
    // super::uio_test::write_to_file(iovecs.as_slice());
}
*/

pub fn test(umem: &Umem, descs: &mut Vec<FrameDesc>, queue: &mut Queue) {
    println!("------t1");
    // pub fn test() {
        // let umem = UMEMINFO.read().unwrap().umem.as_ref().unwrap();
        // let mut descs = UMEMINFO.write().unwrap().descs.as_ref();
    
        // socket
        let mut iovecs = Vec::with_capacity(descs.len());
    
        // get frame
        let write_num = 3;
        for i in 0..write_num {
            let i = queue.get_frame().unwrap();
            println!("---desc=={:?}", i);
            let length = super::uio_test::WRITE_LEN / 3;
            let slice = unsafe { umem.mem.get_data_IoSliceMut(&mut descs[i], length) };
            if i == 0 {
                let buf = "hello world".as_bytes();
                println!("buf={:?}", buf);
                let head_slice = unsafe { umem.mem.get_headroom_IoSliceMut(&mut descs[i], buf.len()) };
                super::uio_test::check_read(&buf, &mut vec![head_slice]);
            }
            // descs[index].adjust_data(length);
            iovecs.push(slice);
        }
    
        println!("iovecs-{}:{:?}", iovecs.len(), iovecs);
    
        // println!("umem = {:?}", umem);
        // println!("umem region = {:?}", umem.mem);
        // println!("descs = {:?}", descs);
    
        super::uio_test::check_read(&super::uio_test::gen_data(), &mut iovecs);
    
        // read
        let mut iovecs = Vec::with_capacity(descs.len());
        let rx_q = vec![0, 1, 2];
        for index in rx_q {
            unsafe {
                println!("descs: {:?}", descs);
                println!(
                    "headroom-{}: {:?}",
                    index,
                    umem.headroom(&descs[index]).contents()
                );
                println!(
                    "data-{}:{}- {:?}",
                    index,
                    umem.data(&descs[index]).contents().len(),
                    umem.data(&descs[index]).contents()
                );
            }
            iovecs.push(unsafe { umem.mem.get_data_IoSlice(&descs[index]) });
        }
        super::uio_test::write_to_file(iovecs.as_slice());
    }
    