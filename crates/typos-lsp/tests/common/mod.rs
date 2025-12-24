use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tower_lsp_server::{LspService, Server};

pub struct TestServer {
    req_client: tokio::io::DuplexStream,
    resp_client: tokio::io::DuplexStream,
    buf: Vec<u8>,
}

impl TestServer {
    pub fn new() -> Self {
        let (req_client, req_server) = tokio::io::duplex(4096);
        let (resp_server, resp_client) = tokio::io::duplex(4096);

        let (service, socket) = LspService::new(typos_lsp::lsp::Backend::new);

        // start server as concurrent task
        tokio::spawn(Server::new(req_server, resp_server, socket).serve(service));

        Self {
            req_client,
            resp_client,
            buf: Vec::new(),
        }
    }

    pub async fn request(&mut self, msg: &str) -> serde_json::Value {
        tracing::debug!("{}", msg);
        let msg = format!("Content-Length: {}\r\n\r\n{}", msg.len(), msg);

        self.req_client.write_all(msg.as_bytes()).await.unwrap();
        self.read_message().await
    }

    pub async fn read_message(&mut self) -> serde_json::Value {
        loop {
            // If a complete message is available in the buffer, return it.
            if let Some((val, len)) = self.try_parse().expect("failed to parse LSP message") {
                self.buf.drain(..len);
                return val;
            }

            // Otherwise read data from the server. The server may return one or more messages.
            // `read_buf` appends to the buffer, each time reading n bytes where n is the buffer's capacity.
            // the buffer will grow as needed.
            // incomplete messages will continue to be read in the next iteration of the loop
            let n = self
                .resp_client
                .read_buf(&mut self.buf)
                .await
                .expect("failed to read from server");

            if n == 0 {
                panic!("server closed connection unexpectedly");
            }
        }
    }

    fn try_parse(&self) -> anyhow::Result<Option<(serde_json::Value, usize)>> {
        // each message is separated by a Content-Length header
        // parse the headers to get the Content-Length and return
        // the first complete message if the buffer contains enough data.
        let mut dst = [httparse::EMPTY_HEADER; 2];

        let (headers_len, headers) = match httparse::parse_headers(&self.buf, &mut dst)? {
            httparse::Status::Complete(val) => val,
            httparse::Status::Partial => return Ok(None), // empty buffer, requires a read first
        };

        let content_length = headers
            .iter()
            .find(|h| h.name.eq_ignore_ascii_case("Content-Length"))
            .ok_or_else(|| anyhow::anyhow!("missing Content-Length header"))?
            .value;

        let content_length: usize = std::str::from_utf8(content_length)?
            .parse()
            .map_err(|e| anyhow::anyhow!("failed to parse Content-Length: {e}"))?;

        let total_len = headers_len + content_length;

        if self.buf.len() < total_len {
            // buffer lacks capacity to contain the entire message, requires another read
            return Ok(None);
        }

        // Parse the message body as JSON. This normalizes key order, making it
        // easier to compare the result in tests.
        let val = serde_json::from_slice(&self.buf[headers_len..total_len])?;
        Ok(Some((val, total_len)))
    }
}
