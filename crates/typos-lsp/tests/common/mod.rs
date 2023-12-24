use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tower_lsp::{LspService, Server};

pub struct TestServer {
    req_client: tokio::io::DuplexStream,
    resp_client: tokio::io::DuplexStream,
    buf: Vec<u8>,
}

impl TestServer {
    pub fn new() -> Self {
        let (req_client, req_server) = tokio::io::duplex(1024);
        let (resp_server, resp_client) = tokio::io::duplex(1024);

        let (service, socket) = LspService::new(typos_lsp::lsp::Backend::new);

        // start server as concurrent task
        tokio::spawn(Server::new(req_server, resp_server, socket).serve(service));

        Self {
            req_client,
            resp_client,
            buf: vec![0; 1024],
        }
    }

    pub async fn request(&mut self, msg: &str) -> &str {
        tracing::debug!("{}", msg);
        let msg = format!("Content-Length: {}\r\n\r\n{}", msg.len(), msg);

        self.req_client.write_all(msg.as_bytes()).await.unwrap();
        let n = self.resp_client.read(&mut self.buf).await.unwrap();

        body(&self.buf[..n]).unwrap()
    }
}

fn body(src: &[u8]) -> Result<&str, anyhow::Error> {
    // parse headers to get headers length
    let mut dst = [httparse::EMPTY_HEADER; 2];

    let (headers_len, _) = match httparse::parse_headers(src, &mut dst)? {
        httparse::Status::Complete(output) => output,
        httparse::Status::Partial => return Err(anyhow::anyhow!("partial headers")),
    };

    // skip headers
    let skipped = &src[headers_len..];

    // return the rest (ie: the body) as &str
    std::str::from_utf8(skipped).map_err(anyhow::Error::from)
}
