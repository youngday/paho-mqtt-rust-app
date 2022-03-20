//use std::string;

use async_std::{io::Result, net::UdpSocket};

pub async fn udp_send(udp2serial_send_ip: &str, send_str: String) -> Result<String> {
    let socket = UdpSocket::bind("127.0.0.1:8082").await?; //TODO:just for creating new socket ,another port
                                                           // info!("Listening on {}", socket.local_addr()?);
    let msg = send_str;
    //info!("<-udp_send:{}", msg);
    socket.send_to(msg.as_bytes(), udp2serial_send_ip).await?;

    Ok(msg)
}

pub async fn udp_received(udp2serial_receive_ip: &str) -> Result<String> {
    let socket = UdpSocket::bind(udp2serial_receive_ip).await?;
    info!("Listening on {}", socket.local_addr()?);

    let mut buf = vec![0u8; 1024];
    let (n, _) = socket.recv_from(&mut buf).await?;
    let received_str = String::from_utf8_lossy(&buf[..n]).to_string();
    info!("->serial to udp:{:?}\n", received_str);

    Ok(received_str)
}
