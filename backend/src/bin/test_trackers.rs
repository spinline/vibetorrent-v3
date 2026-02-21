use shared::xmlrpc::{RtorrentClient, RpcParam, parse_multicall_response};

#[tokio::main]
async fn main() {
    let mut client = None;
    for path in ["127.0.0.1:8000", "0.0.0.0:8000", "localhost:8000"] {
        let test_client = RtorrentClient::new(path);
        match test_client.call("system.client_version", &[]).await {
            Ok(res) => {
                println!("SUCCESS: Connected to rTorrent at {} (Version: {})", path, res);
                client = Some(test_client);
                break;
            }
            Err(_) => {
                // println!("Failed to connect to {}", path);
            }
        }
    }

    let client = match client {
        Some(c) => c,
        None => {
            println!("Could not connect to rTorrent on port 8000.");
            return;
        }
    };

    let mut hash = String::new();
    match client.call("d.multicall2", &[RpcParam::from(""), RpcParam::from("main"), RpcParam::from("d.hash=")]).await {
        Ok(xml) => {
            if let Ok(rows) = parse_multicall_response(&xml) {
                if let Some(row) = rows.first() {
                    if let Some(h) = row.first() {
                        hash = h.clone();
                        println!("Using torrent hash: {}", hash);
                    }
                }
            }
        },
        Err(e) => {
            println!("Error getting torrents: {:?}", e);
            return;
        },
    }

    if hash.is_empty() {
        println!("No torrents found to test trackers.");
        return;
    }

    // Now test Tracker fields one by one to see which one is failing
    let fields = vec![
        "t.url=",
        "t.is_enabled=",
        "t.group=",
        "t.scrape_complete=",
        "t.scrape_incomplete=",
        "t.scrape_downloaded=",
        "t.activity_date_last=",
        "t.normal_interval=",
        "t.message=",
    ];

    for field in &fields {
        let params = vec![
            RpcParam::from(hash.as_str()),
            RpcParam::from(""),
            RpcParam::from(*field),
        ];

        print!("Testing field {:<22} : ", field);
        match client.call("t.multicall", &params).await {
            Ok(xml) => {
                if xml.contains("faultCode") {
                    println!("FAILED");
                } else {
                    println!("SUCCESS");
                }
            },
            Err(e) => println!("ERROR: {:?}", e),
        }
    }
}
