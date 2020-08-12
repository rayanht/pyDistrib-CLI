#[macro_use]
extern crate fstrings;
#[macro_use]
extern crate lazy_static;
extern crate socket2;

use std::io;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, UdpSocket};
use std::time::Duration;

use socket2::{Domain, Protocol, Socket, Type};
use uuid::Uuid;

pub const PORT: u16 = 5007;
lazy_static! {
    pub static ref IPV4: IpAddr = Ipv4Addr::new(224, 1, 1, 1).into();
    pub static ref IPV6: IpAddr = Ipv6Addr::new(0xFF02, 0, 0, 0, 0, 0, 0, 0x0123).into();
}

fn new_socket(addr: &SocketAddr) -> io::Result<Socket> {
    let domain = if addr.is_ipv4() {
        Domain::ipv4()
    } else {
        Domain::ipv6()
    };

    let socket = Socket::new(domain, Type::dgram(), Some(Protocol::udp()))?;

    socket.set_read_timeout(Some(Duration::from_secs(30)))?;

    Ok(socket)
}

#[cfg(windows)]
fn bind_multicast(socket: &Socket, addr: &SocketAddr) -> io::Result<()> {
    let addr = match *addr {
        SocketAddr::V4(addr) => SocketAddr::new(Ipv4Addr::new(0, 0, 0, 0).into(), addr.port()),
        SocketAddr::V6(addr) => {
            SocketAddr::new(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0).into(), addr.port())
        }
    };
    socket.bind(&socket2::SockAddr::from(addr))
}

#[cfg(unix)]
fn bind_multicast(socket: &Socket, addr: &SocketAddr) -> io::Result<()> {
    socket.bind(&socket2::SockAddr::from(*addr))
}

fn subscribe_to_multicast(addr: SocketAddr) -> io::Result<UdpSocket> {
    let ip_addr = addr.ip();

    let socket = new_socket(&addr)?;

    match ip_addr {
        IpAddr::V4(ref mdns_v4) => {
            socket.join_multicast_v4(mdns_v4, &Ipv4Addr::new(0, 0, 0, 0))?;
        }
        IpAddr::V6(ref mdns_v6) => {
            socket.join_multicast_v6(mdns_v6, 0)?;
            socket.set_only_v6(true)?;
        }
    };

    bind_multicast(&socket, &addr)?;
    Ok(socket.into_udp_socket())
}

fn main() {
    let worker_id = Uuid::new_v4();

    println!("Booting worker id {}", worker_id);
    let multicast_socket = SocketAddr::new(*IPV4, PORT);

    // TODO ipv6
    let handshake_socket = UdpSocket::bind(SocketAddr::from(([0, 0, 0, 0], 0)))
        .expect("Failed to init handshake socket");

    let listener =
        subscribe_to_multicast(multicast_socket).expect("Failed to create listener socket");

    let mut buf = [0u8; 128];

    loop {
        match listener.recv_from(&mut buf) {
            Ok((len, remote_addr)) => {
                let data = &buf[..len];
                let decoded_data = String::from_utf8_lossy(data);

                if &decoded_data[..14] == "PyDistrib INIT" {
                    let handshake_address = SocketAddr::new(
                        remote_addr.ip(),
                        *&decoded_data[15..].trim().parse::<u16>().unwrap(),
                    );

                    let payload = f!("PyDistrib HANDSHAKE|{worker_id}");
                    handshake_socket
                        .send_to(payload.as_bytes(), handshake_address)
                        .expect("Failed to handshake");

                    match handshake_socket.recv_from(&mut buf) {
                        Ok((len, remote_addr)) => {
                            let data = &buf[..len];
                            let decoded_data = String::from_utf8_lossy(data);
                            let expected_data = f!("PyDistrib HANDSHAKE ACK|{worker_id}");

                            if decoded_data.trim() == expected_data
                                && remote_addr.ip() == handshake_address.ip()
                            {
                                println!("Server acknowledged the handshake");
                                break;
                            }
                        }
                        _ => {}
                    }
                }
            }
            Err(err) => {
                println!("Worker error: {}", err);
            }
        }
    }
}
