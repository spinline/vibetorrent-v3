use shared::xmlrpc::{RtorrentClient, RpcParam, parse_multicall_response};

#[tokio::main]
async fn main() {
    let client = RtorrentClient::new("/tmp/rtorrent.sock");
    
    // Hardcode a known hash from the UI, e.g. "C3315ABFAD70C54505813D1303C1457900C5B795" (from first image)
    let hash = "C3315ABFAD70C54505813D1303C1457900C5B795";
    
    let params = vec![
        RpcParam::from(hash),
        RpcParam::from(""),
        RpcParam::from("t.url="),
    ];

    match client.call("t.multicall", &params).await {
        Ok(xml) => {
            println!("Response XML:\n{}", xml);
            match parse_multicall_response(&xml) {
                Ok(rows) => println!("Rows ({})", rows.len()),
                Err(e) => println!("Parse error: {:?}", e),
            }
        },
        Err(e) => println!("Error: {:?}", e),
    }
}
