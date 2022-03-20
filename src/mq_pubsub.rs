//***********************************
// paho-mqtt/examples/topic_publish.rs
// paho-mqtt/examples/async_subscribe.rs
// use the await async fun v3.1
//-------------------------------------

mod udp;
use async_std::{stream::StreamExt, task};
use std::{io, io::Read, process, time::Duration};

use paho_mqtt as mqtt;

const TOPIC_RCV_UDP: &str = "rcv_udp";
const TOPIC_SEND_UDP: &str = "send_udp";
const TOPICS_SUB_UDP: &[&str] = &["test_udp", TOPIC_SEND_UDP];
const QOS_SUB_UDP: &[i32] = &[1, 1];
const QOS_PUB_UDP: i32 = 1;

const TOPIC_RCV_SERIAL: &str = "rcv_serial";
const TOPIC_SEND_SERIAL: &str = "send_serial";
const TOPICS_SUB_SERIAL: &[&str] = &["test_serial", TOPIC_SEND_SERIAL];
const QOS_SUB_SERIAL: &[i32] = &[1, 1];
const QOS_PUB_SERIAL: i32 = 1;

pub async fn udp_pub(mqtt_ip: String, udp2serial_receive_ip: String) {
    // Create a client & define connect options
    let cli = mqtt::AsyncClient::new(mqtt_ip).unwrap_or_else(|err| {
        error!("Error creating the client: {}", err);
        process::exit(1);
    });

    let conn_opts = mqtt::ConnectOptions::new();

    // Connect and wait for it to complete or fail
    if let Err(e) = cli.connect(conn_opts).wait() {
        error!("Unable to connect: {:?}", e);
        process::exit(1);
    }

    // Create a topic and publish to it
    info!("Publishing messages on the {:?} topic", TOPIC_RCV_UDP);
    let topic = mqtt::Topic::new(&cli, TOPIC_RCV_UDP, QOS_PUB_UDP);

    loop {
        let received_ip = udp2serial_receive_ip.as_str();
        let result = udp::udp_received(received_ip).await;
        match result {
            Ok(v) => {
                info!("pub:{:?}:{:?}", TOPIC_RCV_UDP, v);
                let tok = topic.publish(v);
                if let Err(e) = tok.wait() {
                    error!("Error: pub:{:?}:{:?}", TOPIC_RCV_UDP, e);
                    //  break;
                }
            }
            Err(e) => error!("Error: udp_received error {:?}.",e),
        }
    }
}

pub async fn udp_sub(mqtt_ip: String, udp2serial_send_ip: String) {
    // Create the client. Use an ID for a persistent session.
    // A real system should try harder to use a unique ID.
    let create_opts = mqtt::CreateOptionsBuilder::new()
        .server_uri(mqtt_ip)
        .client_id("rust_async_subscribe")
        .finalize();

    // Create the client connection
    let mut cli = mqtt::AsyncClient::new(create_opts).unwrap_or_else(|e| {
        error!("Error creating the client: {:?}", e);
        process::exit(1);
    });

    if let Err(err) = task::block_on(async move {
        // Get message stream before connecting.
        let mut strm = cli.get_stream(256); //orign 25

        // Define the set of options for the connection
        let lwt = mqtt::Message::new(
            TOPIC_SEND_UDP,
            "Async subscriber lost connection",
            mqtt::QOS_1,
        );

        let conn_opts = mqtt::ConnectOptionsBuilder::new()
            .keep_alive_interval(Duration::from_secs(30))
            .mqtt_version(mqtt::MQTT_VERSION_3_1_1)
            .clean_session(false)
            .will_message(lwt)
            .finalize();

        // Make the connection to the broker
        info!("Connecting to the MQTT server...");
        cli.connect(conn_opts).await?;

        info!("Subscribing to topics: {:?}", TOPICS_SUB_UDP);
        cli.subscribe_many(TOPICS_SUB_UDP, QOS_SUB_UDP).await?;

        // Just loop on incoming messages.
        info!("Waiting for messages...");

        // Note that we're not providing a way to cleanly shut down and
        // disconnect. Therefore, when you kill this app (with a ^C or
        // whatever) the server will get an unexpected drop and then
        // should emit the LWT message.

        let send_ip = udp2serial_send_ip.as_str();
        while let Some(msg_opt) = strm.next().await {
            if let Some(msg) = msg_opt {
                info!("sub_topic:{:?}", msg.topic());
                info!("sub_paload:{:?}", msg.payload_str());
                //received sub msg,and then send it to udp

                let send_str = msg.payload_str().to_string();
                info!("sub:{:?}:{:?}", TOPIC_SEND_UDP, send_str);

                match udp::udp_send(send_ip, send_str).await {
                    Ok(v) => {
                        info!("udp_send ok:{:?}", v);
                    }
                    Err(e) => error!("Error:udp_send:{:?}", e),
                };
            } else {
                // A "None" means we were disconnected. Try to reconnect...
                info!("Lost connection. Attempting reconnect.");
                while let Err(err) = cli.reconnect().await {
                    error!("Error reconnecting: {}", err);
                    // For tokio use: tokio::time::delay_for()
                    async_std::task::sleep(Duration::from_millis(1000)).await;
                }
            }
        }

        // Explicit return type for the async block
        Ok::<(), mqtt::Error>(())
    }) {
        error!("{}", err);
    }
}

pub async fn serial_port(port_name: String, baud_rate: u32) {
    //++++++++++++++++++serial port+++++++++++++++++++++++++++++++++++++++++++
    let port = serialport::new(port_name.clone(), baud_rate)
        .timeout(Duration::from_millis(10))
        .open();

    match port {
        Ok(mut port) => {
            // Clone the port
            let mut port_write_clone = port.try_clone().expect("Failed to clone");

            // Send out 4 bytes every second
            task::spawn(async move {
                // Create the client. Use an ID for a persistent session.
                // A real system should try harder to use a unique ID.
                let mqtt_ip = "tcp://127.0.0.1:1883".to_string();
                let create_opts = mqtt::CreateOptionsBuilder::new()
                    .server_uri(mqtt_ip)
                    .client_id("rust_async_subscribe")
                    .finalize();

                // Create the client connection
                let mut cli = mqtt::AsyncClient::new(create_opts).unwrap_or_else(|e| {
                    error!("Error creating the client: {:?}", e);
                    process::exit(1);
                });

                if let Err(err) = task::block_on(async move {
                    // Get message stream before connecting.
                    let mut strm = cli.get_stream(256); //orign 25
                    // Define the set of options for the connection
                    let lwt = mqtt::Message::new(
                        TOPIC_SEND_SERIAL,
                        "Async subscriber lost connection",
                        mqtt::QOS_1,
                    );
                    let conn_opts = mqtt::ConnectOptionsBuilder::new()
                        .keep_alive_interval(Duration::from_secs(30))
                        .mqtt_version(mqtt::MQTT_VERSION_3_1_1)
                        .clean_session(false)
                        .will_message(lwt)
                        .finalize();
                    // Make the connection to the broker
                    info!("Connecting to the MQTT server...");
                    cli.connect(conn_opts).await?;
                    info!("Subscribing to topics: {:?}", TOPICS_SUB_SERIAL);
                    cli.subscribe_many(TOPICS_SUB_SERIAL, QOS_SUB_SERIAL).await?;
                    // Just loop on incoming messages.
                    info!("Waiting for messages...");
                    // Note that we're not providing a way to cleanly shut down and
                    // disconnect. Therefore, when you kill this app (with a ^C or
                    // whatever) the server will get an unexpected drop and then
                    // should emit the LWT message.
                    while let Some(msg_opt) = strm.next().await {
                        if let Some(msg) = msg_opt {
                            info!("sub_topic:{:?}", msg.topic());
                            info!("sub_paload:{:?}", msg.payload_str());
                            //received sub msg,and then send it to serial
                            let send_str = msg.payload_str().to_string();
                            info!("sub:{:?}:{:?}", TOPIC_SEND_SERIAL, send_str);

                            let output = send_str.as_str();
                            port_write_clone
                                .write_all(output.as_bytes())
                                .expect("Failed to write to serial port");
                          
                        } else {
                            // A "None" means we were disconnected. Try to reconnect...
                            info!("Lost connection. Attempting reconnect.");
                            while let Err(err) = cli.reconnect().await {
                                error!("Error reconnecting: {}", err);
                                // For tokio use: tokio::time::delay_for()
                                async_std::task::sleep(Duration::from_millis(1000)).await;
                            }
                        }
                    }
                    // Explicit return type for the async block
                    Ok::<(), mqtt::Error>(())
                }) {
                    error!("{}", err);
                }
            });

            let mut serial_buf: Vec<u8> = vec![0; 1000];
            info!(
                "Receiving data on {:?} at {:?} baud:",
                port_name.clone(),
                baud_rate
            );

            task::spawn(async move {
                // Create a client & define connect options
                let mqtt_ip = "tcp://127.0.0.1:1883".to_string();
                let cli = mqtt::AsyncClient::new(mqtt_ip).unwrap_or_else(|err| {
                    error!("Error creating the client: {}", err);
                    process::exit(1);
                });

                let conn_opts = mqtt::ConnectOptions::new();

                // Connect and wait for it to complete or fail
                if let Err(e) = cli.connect(conn_opts).wait() {
                    error!("Unable to connect: {:?}", e);
                    process::exit(1);
                }

                // Create a topic and publish to it
                info!("Publishing messages on the {:?} topic", TOPIC_RCV_SERIAL);
                let topic = mqtt::Topic::new(&cli, TOPIC_RCV_SERIAL, QOS_PUB_SERIAL);

                loop {
                    //TODO:sync serial ,change into async serial,mio-serial,have no bidirection port.
                    match port.read(serial_buf.as_mut_slice()) {
                        Ok(v) => {
                            // io::stdout().write_all(&serial_buf[..v]).unwrap();
                            let out_string = String::from_utf8(serial_buf[..v].to_vec()).unwrap();

                            info!("Receiving data  {:?}", out_string);
                            info!("pub:{:?}:{:?}", TOPIC_RCV_SERIAL, out_string);
                            let tok = topic.publish(out_string);
                            if let Err(e) = tok.wait() {
                                error!("Error: pub:{:?}:{:?}", TOPIC_RCV_SERIAL, e);
                                //  break;
                            }
                        }
                        Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                        Err(e) => error!("{:?}", e),
                    }
                }
            });
        }
        Err(e) => {
            error!("Failed to open \"{}\". Error: {}", port_name.clone(), e);
            ::std::process::exit(1);
        }
    }
    //-------------------serial port--------------------------------
}
