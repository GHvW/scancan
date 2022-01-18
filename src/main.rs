use std::error::Error;
use std::format;
use tokio::sync::mpsc;
use tokio::net::TcpStream;
use async_channel;



#[tokio::main]
async fn main() {
    println!("Scan Ports!");

    let (ports_tx, ports_rx) = async_channel::bounded(100);
    let (results_tx, mut results_rx) = mpsc::channel(1);

    (1..100)
        .for_each(move |_| {
            let ports_receiver = ports_rx.clone();
            let results_sender = results_tx.clone();
            tokio::spawn(async move { 
                worker(ports_receiver, results_sender).await.expect("something went wrong with a worker");
            });
        });

    tokio::spawn(async move {
        for i in 1..=1024 {
            ports_tx.send(i).await.expect("something went wrong sending port numbers");
        }

        ports_tx.close();
    });

    let mut results = Vec::new();
    while let Some(msg) = results_rx.recv().await {
        if msg != 0 {
            // println!("port: {} is open", msg);
            results.push(msg);
        }
    }

    println!("ports {:?} are open", results);
}

async fn worker(ports: async_channel::Receiver<i32>, results: mpsc::Sender<i32>) -> Result<(), Box<dyn Error>> {
    let address = "scanme.nmap.org";

    while let Ok(port) = ports.recv().await {
        // println!("checking port {}", port);
        match TcpStream::connect(format!("{}:{}", address, port)).await {
            Ok(_) => {
                results.send(port).await?;
            },
            Err(_) => {
                results.send(0).await?;
            }
        }
    }

    Ok(())
}