use tokio::net::UdpSocket;
#[cfg(unix)]
use std::os::unix::io::{AsRawFd, RawFd};


pub async fn socket_test() -> std::io::Result<()> {
    let sock = UdpSocket::bind("0.0.0.0:8080").await?;
    // let sock = sock.into_std()?;
    // sock.set_nonblocking(true)?;

    let fd = sock.as_raw_fd();
    println!("socket fd = {}", fd);



    let mut buf = [0; 1024];
    // loop {
    //     let (len, addr) = sock.recv_from(&mut buf).await?;
    //     println!("{:?} bytes received from {:?}", len, addr);

    //     let len = sock.send_to(&buf[..len], addr).await?;
    //     println!("{:?} bytes sent", len);
    // }
    Ok(())
}
