use std::io;

use tokio::net::TcpListener;

async fn get() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    loop {
        let (socket, _) = listener.accept().await?;
        process_socket(socket).await;
    }
}
