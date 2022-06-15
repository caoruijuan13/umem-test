use nix::sys::uio::*;
use nix::unistd::*;
use std::{cmp, iter};
use std::fs::{OpenOptions};
use std::os::unix::io::AsRawFd;
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

use std::vec;

#[cfg(not(target_os = "redox"))]
use std::io::{IoSlice, IoSliceMut};

#[cfg(not(target_os = "redox"))]
use tempfile::tempfile;
use tempfile::tempdir;

pub fn test_readv() {
    let to_write = gen_data();
    println!("to_write:{:?}", to_write);
    let mut storage = Vec::new();
    let mut allocated = 0;
    while allocated < to_write.len() {
        let left = to_write.len() - allocated;
        let vec_len = if left <= 64 { left } else { thread_rng().gen_range(64..cmp::min(256, left)) };
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
    check_read(&mut iovecs, to_write);
}

pub fn test_pwritev() {
    use std::io::Read;

    let to_write: Vec<u8> = (0..128).collect();
    let expected: Vec<u8> = [vec![0;100], to_write.clone()].concat();

    let iovecs = [
        IoSlice::new(&to_write[0..17]),
        IoSlice::new(&to_write[17..64]),
        IoSlice::new(&to_write[64..128]),
    ];

    let tempdir = tempdir().unwrap();

    // pwritev them into a temporary file
    let path = tempdir.path().join("pwritev_test_file");
    let mut file = OpenOptions::new().write(true).read(true).create(true)
                                    .truncate(true).open(path).unwrap();

    let written = pwritev(file.as_raw_fd(), &iovecs, 100).ok().unwrap();
    println!("written={:?}\n-{}", to_write, written );
    // assert_eq!(written, to_write.len());

    // Read the data back and make sure it matches
    let mut contents = Vec::new();
    file.read_to_end(&mut contents).unwrap();
    println!("read={:?}\n-{:?}", contents, expected );
    assert_eq!(contents, expected);
}

/// generata data
pub fn gen_data() -> Vec<u8> {
    let s:String = thread_rng()
        .sample_iter(&Alphanumeric)
        .map(char::from)
        .take(128)
        .collect();
    let to_write = s.as_bytes().to_vec();
    println!("to_write:{:?}", to_write);
    to_write
}

/// check read
pub fn check_read(iovecs: &mut Vec<IoSliceMut>, to_write: Vec<u8>) {
    let pipe_res = pipe();
    assert!(pipe_res.is_ok());
    let (reader, writer) = pipe_res.ok().unwrap();
    // Blocking io, should write all data.
    let write_res = write(writer, &to_write);
    // Successful write
    println!("write_res:{:?}", write_res);
    assert!(write_res.is_ok());
    let read_res = readv(reader, &mut iovecs[..]);
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