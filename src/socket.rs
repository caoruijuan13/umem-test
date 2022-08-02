/// Wrapper for a raw Linux socket

use std::io::{self, Error, ErrorKind, Read, Write};
use std::os::unix::io::{AsRawFd, RawFd};
use std::net::UdpSocket;

// use libc;

pub fn init_sock() -> RawFd {
    let sock = UdpSocket::bind("127.0.0.1:8080").unwrap();
    // let sock = sock.into_std()?;
    sock.set_nonblocking(true).unwrap();

    let fd = sock.as_raw_fd();
    println!("socket fd = {}", fd);
    fd
}

/*
#[derive(Debug, Clone, Copy)]
pub struct Socket {
    fd: RawFd
}

impl Socket {
    pub fn new(fd: RawFd) -> Socket { Socket { fd: fd } }

    pub fn recv(&self, buf: &mut [u8]) -> io::Result<usize> {
        let l = buf.len();
        let b = buf.as_mut_ptr() as *mut libc::c_void;
        let r = unsafe { libc::recv(self.fd, b, l, 0) };

        // These calls return the number of bytes received, or -1 if an error
        // occurred.  In the event of an error, errno is set to indicate the
        // error.
        if r == -1 {
            Err(Error::last_os_error())
        } else if r == 0 {
            // When a stream socket peer has performed an orderly shutdown, the
            // return value will be 0 (the traditional "end-of-file" return).
            //
            // The value 0 may also be returned if the requested number of bytes
            // to receive from a stream socket was 0.
            if buf.len() == 0 { Ok(0) }
            else { Err(Error::new(ErrorKind::UnexpectedEof, "EOF")) }
        } else {
            Ok(r as usize)
        }
    }

    pub fn send(&self, buf: &[u8]) -> io::Result<usize> {
        let l = buf.len();
        let b = buf.as_ptr() as *const libc::c_void;
        let r = unsafe { libc::send(self.fd, b, l, 0) };

        // On success, these calls return the number of bytes sent.
        // On error, -1 is returned, and errno is set appropriately.
        if r == -1 {
            Err(Error::last_os_error())
        } else if r == 0 {
            if buf.len() == 0 { Ok(0) }
            else { Err(Error::new(ErrorKind::WriteZero, "WriteZero")) }
        } else {
            Ok(r as usize)
        }
    }
}

impl AsRawFd for Socket { fn as_raw_fd(&self) -> RawFd { self.fd } }

impl Read for Socket {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> { self.recv(buf) }
}

impl Write for Socket {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> { self.send(buf) }
    fn flush(&mut self) -> io::Result<()> { Ok(()) }
}
*/