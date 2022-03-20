mod mq_pubsub;

#[macro_use]
extern crate log;
use async_std::{channel, task};
use log::{debug, error, info, trace, warn};
use log4rs;
use serde::{Deserialize, Serialize};
use std::{error::Error, fs::File, time::Duration};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Application {
    app: Data,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Data {
    build: String,
    container_name: String,
    ip_config: IPCONFIG,
    serial_port: PortParam,
    #[serde(skip_serializing_if = "Option::is_none")]
    environment: Option<Vec<String>>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct IPCONFIG {
    mqtt_ip: String,
    udp2serial_send_ip: String,
    udp2serial_receive_ip: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct PortParam {
    port_no: String,
    baudrate: u32,
}

#[async_std::main] //async-std = { version = "1.9", features = ["attributes"] }
async fn main() -> Result<(), Box<dyn Error>> {
    log4rs::init_file("./log.yaml", Default::default()).unwrap();
    let version: String = "0.2.0910".to_string();
    trace!("demo trace");
    debug!("demo debug");
    info!("demo info");
    warn!("demo warn");
    error!("demo error");

    info!("version:{:}", version);
    let param_file_path: &str = "config.yaml";
    let param_file = File::open(&param_file_path)?; //sync for parameter reading.
    let app_data: Application = serde_yaml::from_reader(param_file)?;

    let port_no: String = app_data.app.serial_port.port_no;
    let baudrate: u32 = app_data.app.serial_port.baudrate;
    let mqtt_ip_pub: String = app_data.app.ip_config.mqtt_ip.clone();
    let mqtt_ip_sub: String = app_data.app.ip_config.mqtt_ip.clone();
    let udp2serial_send_ip: String = app_data.app.ip_config.udp2serial_send_ip.clone();
    let udp2serial_receive_ip: String = app_data.app.ip_config.udp2serial_receive_ip.clone();

    info!("{:?}", port_no);
    info!("{:?}", baudrate);
    info!("{:?}", mqtt_ip_pub);
    info!("{:?}", mqtt_ip_sub);
    info!("{:?}", udp2serial_send_ip);
    info!("{:?}", udp2serial_receive_ip);

    info!("Start your app.");

    let (tx, rx) = channel::bounded(1);
    let r = task::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            info!("Received: {}", msg);
        }
    });

    let t = task::spawn(async move {
        loop {
            tx.send("Channel test!").await.unwrap();
            task::sleep(Duration::from_millis(5000)).await;
        }
    });

    let udp_pub = task::spawn(async move {
        task::Builder::new()
            .name("udp_pub".to_string())
            .spawn(mq_pubsub::udp_pub(mqtt_ip_pub, udp2serial_receive_ip))
            .unwrap()
            .await;
    });

    let udp_sub = task::spawn(async move {
        task::Builder::new()
            .name("udp_sub".to_string())
            .spawn(mq_pubsub::udp_sub(mqtt_ip_sub, udp2serial_send_ip))
            .unwrap()
            .await;
    });
    // let serial_port = task::spawn(async move {
    //     task::Builder::new()
    //         .name("serial_port".to_string())
    //         .spawn(mq_pubsub::serial_port(port_no, baudrate))
    //         .unwrap()
    //         .await;
    // });

    let spawn_test = task::spawn(async {
        task::Builder::new()
            .name("spawn_test".to_string())
            .spawn(spawn_test())
            .unwrap()
            .await;
    });

    let block_loop = task::spawn(async {//pre task::block_on for sync main
        //if we use async main,need not block_on here,reserved
        task::Builder::new()
            .name("block_loop".to_string())
            .spawn(main_loop())
            .unwrap()
            .await;
    });

    t.await;
    r.await;
    udp_pub.await;
    udp_sub.await;
    //serial_port.await;
    spawn_test.await;
    block_loop.await;

    Ok(())
}

pub async fn main_loop() {
    info!("entry into main_loop .");
    loop {
        info!("main looping.");
        task::sleep(Duration::from_millis(5000)).await;
    }
}

pub async fn spawn_test() {
    loop {
        info!("task spawn_test!");
        task::sleep(Duration::from_millis(3000)).await;
    }
}
