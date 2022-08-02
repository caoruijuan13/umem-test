use nix::sys::uio::*;
use nix::unistd::*;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use std::fs::{File, OpenOptions};
use std::os::unix::io::AsRawFd;
use std::{cmp, iter};

use std::vec;

#[cfg(not(target_os = "redox"))]
use std::io::{IoSlice, IoSliceMut};

use tempfile::tempdir;
#[cfg(not(target_os = "redox"))]
use tempfile::tempfile;

use std::io::Read;

pub const WRITE_LEN: usize = 64 * 3;

/// write test
pub fn write_test(buf: &mut [u8]) {
    let mut file = tempfile().unwrap();
    // let buf = [1u8;8];
    assert_eq!(Ok(8), pwrite(file.as_raw_fd(), &buf, 8));
    let mut file_content = Vec::new();
    file.read_to_end(&mut file_content).unwrap();
    let mut expected = vec![0u8; 8];
    expected.extend(vec![1; 8]);
    assert_eq!(file_content, expected);
}

pub fn test_readv() {
    let to_write = gen_data();
    println!("to_write:{:?}", to_write);
    let mut storage = Vec::new();
    let mut allocated = 0;
    while allocated < to_write.len() {
        let left = to_write.len() - allocated;
        let vec_len = if left <= 64 {
            left
        } else {
            thread_rng().gen_range(64..cmp::min(256, left))
        };
        println!("left:{}, vec_len:{:?}", left, vec_len);
        let v: Vec<u8> = iter::repeat(0u8).take(vec_len).collect();
        storage.push(v);
        allocated += vec_len;
    }

    println!("storage:{:?}", storage.len());
    let mut iovecs = Vec::with_capacity(storage.len());
    for v in &mut storage {
        iovecs.push(IoSliceMut::new(&mut v[..]));
    }
    check_read(&gen_data(), &mut iovecs);
}

pub fn test_pwritev() {
    use std::io::Read;

    let to_write: Vec<u8> = (0..128).collect();
    let expected: Vec<u8> = [vec![0; 100], to_write.clone()].concat();

    let iovecs = [
        IoSlice::new(&to_write[0..17]),
        IoSlice::new(&to_write[17..64]),
        IoSlice::new(&to_write[64..128]),
    ];

    let tempdir = tempdir().unwrap();

    // pwritev them into a temporary file
    let path = tempdir.path().join("pwritev_test_file");
    let mut file = OpenOptions::new()
        .write(true)
        .read(true)
        .create(true)
        .truncate(true)
        .open(path)
        .unwrap();

    let written = pwritev(file.as_raw_fd(), &iovecs, 100).ok().unwrap();
    println!("written={:?}\n-{}", to_write, written);
    // assert_eq!(written, to_write.len());

    // Read the data back and make sure it matches
    let mut contents = Vec::new();
    file.read_to_end(&mut contents).unwrap();
    println!("read={:?}\n-{:?}", contents, expected);
    assert_eq!(contents, expected);
}

/// generata data
pub fn gen_data() -> Vec<u8> {
    let s: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .map(char::from)
        .take(WRITE_LEN)
        .collect();
    let to_write = s.as_bytes().to_vec();
    println!("to_write:{:?}", to_write);
    to_write
}

/// check read
pub fn check_read(to_write: &[u8], iovecs: &mut Vec<IoSliceMut>) {
    let pipe_res = pipe();
    assert!(pipe_res.is_ok());
    
    let fd = super::socket::init_sock();

    let (reader, writer) = pipe_res.ok().unwrap();
    // Blocking io, should write all data.
    // let write_res = write(writer, to_write);
    let write_res = pwrite(fd, &to_write, to_write.len() as i64);


    // Successful write
    println!("write_res:{:?}", write_res);
    assert!(write_res.is_ok());

    let read_res = readv(fd, &mut iovecs[..]);
    // let read_res = readv(reader, &mut iovecs[..]);
    println!("read_res:{:?}", read_res);
    assert!(read_res.is_ok());
    let read = read_res.ok().unwrap();
    // Check whether we've read all data
    assert_eq!(to_write.len(), read);
    // Cccumulate data from iovecs
    let mut read_buf = Vec::with_capacity(to_write.len());
    for iovec in iovecs {
        read_buf.extend(iovec.iter().cloned());
    }
    // Check whether iovecs contain all written data
    assert_eq!(read_buf.len(), to_write.len());
    // Check equality of written and read data
    println!("read_buf:{:?}", read_buf);
    assert_eq!(&read_buf, &to_write);
    let close_res = close(reader);
    assert!(close_res.is_ok());
    let close_res = close(writer);
    assert!(close_res.is_ok());
}

/// write ioslice to file
pub fn write_to_file(iovecs: &[IoSlice]) {
    // pwritev them into a temporary file
    let mut file = File::create("test.txt").unwrap();
    let written = pwritev(file.as_raw_fd(), iovecs, 0).ok().unwrap();
    // assert_eq!(written, to_write.len());

    // Read the data back and make sure it matches
    // let mut contents = Vec::new();
    // file.read_to_end(&mut contents).unwrap();
    let contents = std::fs::read("test.txt").unwrap();
    println!("read-{}={:?}", contents.len(), contents);
}

/// check write to headroom
pub fn check_write_headroom(buf: &[u8], iovecs: &mut Vec<IoSliceMut>) {
    let mut file = File::create("header.txt").unwrap();
    let buf = "hello world".as_bytes();
    println!("buf={:?}", buf);
    let write_res = pwrite(file.as_raw_fd(), &buf, buf.len() as i64);
    println!("write_res:{:?}", write_res);
    let read_res = readv(file.as_raw_fd(), &mut iovecs[..]);
    println!("read_res:{:?}, {:?}", read_res, iovecs);
}
