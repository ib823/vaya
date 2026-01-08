//! WebSocket implementation

use ring::digest::{digest, SHA1_FOR_LEGACY_USE_ONLY};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::{NetError, NetResult, Request, Response, StatusCode};

/// WebSocket magic GUID for handshake
const WS_MAGIC_GUID: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

/// WebSocket opcode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Opcode {
    Continuation = 0x0,
    Text = 0x1,
    Binary = 0x2,
    Close = 0x8,
    Ping = 0x9,
    Pong = 0xA,
}

impl Opcode {
    fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x0 => Some(Opcode::Continuation),
            0x1 => Some(Opcode::Text),
            0x2 => Some(Opcode::Binary),
            0x8 => Some(Opcode::Close),
            0x9 => Some(Opcode::Ping),
            0xA => Some(Opcode::Pong),
            _ => None,
        }
    }
}

/// A WebSocket message
#[derive(Debug, Clone)]
pub enum Message {
    /// Text message
    Text(String),
    /// Binary message
    Binary(Vec<u8>),
    /// Ping message
    Ping(Vec<u8>),
    /// Pong message
    Pong(Vec<u8>),
    /// Close message
    Close(Option<(u16, String)>),
}

impl Message {
    /// Get the opcode for this message
    pub fn opcode(&self) -> Opcode {
        match self {
            Message::Text(_) => Opcode::Text,
            Message::Binary(_) => Opcode::Binary,
            Message::Ping(_) => Opcode::Ping,
            Message::Pong(_) => Opcode::Pong,
            Message::Close(_) => Opcode::Close,
        }
    }

    /// Get the payload bytes
    pub fn payload(&self) -> Vec<u8> {
        match self {
            Message::Text(s) => s.as_bytes().to_vec(),
            Message::Binary(b) => b.clone(),
            Message::Ping(b) => b.clone(),
            Message::Pong(b) => b.clone(),
            Message::Close(Some((code, reason))) => {
                let mut data = code.to_be_bytes().to_vec();
                data.extend_from_slice(reason.as_bytes());
                data
            }
            Message::Close(None) => Vec::new(),
        }
    }

    /// Check if this is a control frame
    pub fn is_control(&self) -> bool {
        matches!(self, Message::Ping(_) | Message::Pong(_) | Message::Close(_))
    }
}

/// WebSocket connection
pub struct WebSocket<S> {
    stream: S,
    closed: bool,
}

impl<S> WebSocket<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    /// Create a WebSocket from an existing stream after handshake
    pub fn new(stream: S) -> Self {
        Self {
            stream,
            closed: false,
        }
    }

    /// Create the WebSocket handshake response
    pub fn handshake_response(request: &Request) -> NetResult<Response> {
        let key = request
            .websocket_key()
            .ok_or_else(|| NetError::WebSocket("Missing Sec-WebSocket-Key".into()))?;

        let accept = Self::compute_accept_key(key);

        Ok(Response::new(StatusCode::SwitchingProtocols)
            .header("Upgrade", "websocket")
            .header("Connection", "Upgrade")
            .header("Sec-WebSocket-Accept", accept))
    }

    /// Compute the Sec-WebSocket-Accept header value
    fn compute_accept_key(key: &str) -> String {
        let combined = format!("{}{}", key, WS_MAGIC_GUID);
        let hash = digest(&SHA1_FOR_LEGACY_USE_ONLY, combined.as_bytes());
        base64_encode(hash.as_ref())
    }

    /// Read a message from the WebSocket
    pub async fn read(&mut self) -> NetResult<Message> {
        if self.closed {
            return Err(NetError::ConnectionClosed);
        }

        let frame = self.read_frame().await?;

        match frame.opcode {
            Opcode::Text => {
                let text = String::from_utf8(frame.payload).map_err(|_| {
                    NetError::WebSocket("Invalid UTF-8 in text frame".into())
                })?;
                Ok(Message::Text(text))
            }
            Opcode::Binary => Ok(Message::Binary(frame.payload)),
            Opcode::Ping => Ok(Message::Ping(frame.payload)),
            Opcode::Pong => Ok(Message::Pong(frame.payload)),
            Opcode::Close => {
                self.closed = true;
                if frame.payload.len() >= 2 {
                    let code = u16::from_be_bytes([frame.payload[0], frame.payload[1]]);
                    let reason = String::from_utf8_lossy(&frame.payload[2..]).to_string();
                    Ok(Message::Close(Some((code, reason))))
                } else {
                    Ok(Message::Close(None))
                }
            }
            Opcode::Continuation => {
                Err(NetError::WebSocket("Unexpected continuation frame".into()))
            }
        }
    }

    /// Write a message to the WebSocket
    pub async fn write(&mut self, message: Message) -> NetResult<()> {
        if self.closed && !matches!(message, Message::Close(_)) {
            return Err(NetError::ConnectionClosed);
        }

        let frame = Frame {
            fin: true,
            opcode: message.opcode(),
            mask: None, // Server frames are not masked
            payload: message.payload(),
        };

        self.write_frame(&frame).await
    }

    /// Send a text message
    pub async fn send_text(&mut self, text: impl Into<String>) -> NetResult<()> {
        self.write(Message::Text(text.into())).await
    }

    /// Send a binary message
    pub async fn send_binary(&mut self, data: impl Into<Vec<u8>>) -> NetResult<()> {
        self.write(Message::Binary(data.into())).await
    }

    /// Send a ping
    pub async fn ping(&mut self, data: impl Into<Vec<u8>>) -> NetResult<()> {
        self.write(Message::Ping(data.into())).await
    }

    /// Send a pong
    pub async fn pong(&mut self, data: impl Into<Vec<u8>>) -> NetResult<()> {
        self.write(Message::Pong(data.into())).await
    }

    /// Close the connection
    pub async fn close(&mut self, code: Option<u16>, reason: Option<&str>) -> NetResult<()> {
        let close_data = code.map(|c| (c, reason.unwrap_or("").to_string()));
        self.write(Message::Close(close_data)).await?;
        self.closed = true;
        Ok(())
    }

    /// Read a WebSocket frame
    async fn read_frame(&mut self) -> NetResult<Frame> {
        // Read first two bytes
        let mut header = [0u8; 2];
        self.stream.read_exact(&mut header).await?;

        let fin = (header[0] & 0x80) != 0;
        let opcode = Opcode::from_u8(header[0] & 0x0F)
            .ok_or_else(|| NetError::WebSocket("Invalid opcode".into()))?;
        let masked = (header[1] & 0x80) != 0;
        let mut payload_len = (header[1] & 0x7F) as u64;

        // Extended payload length
        if payload_len == 126 {
            let mut buf = [0u8; 2];
            self.stream.read_exact(&mut buf).await?;
            payload_len = u16::from_be_bytes(buf) as u64;
        } else if payload_len == 127 {
            let mut buf = [0u8; 8];
            self.stream.read_exact(&mut buf).await?;
            payload_len = u64::from_be_bytes(buf);
        }

        // Read mask if present
        let mask = if masked {
            let mut mask_key = [0u8; 4];
            self.stream.read_exact(&mut mask_key).await?;
            Some(mask_key)
        } else {
            None
        };

        // Read payload
        let mut payload = vec![0u8; payload_len as usize];
        self.stream.read_exact(&mut payload).await?;

        // Unmask if needed
        if let Some(mask_key) = mask {
            for (i, byte) in payload.iter_mut().enumerate() {
                *byte ^= mask_key[i % 4];
            }
        }

        Ok(Frame {
            fin,
            opcode,
            mask,
            payload,
        })
    }

    /// Write a WebSocket frame
    async fn write_frame(&mut self, frame: &Frame) -> NetResult<()> {
        let mut header = Vec::new();

        // First byte: FIN + opcode
        let first_byte = if frame.fin { 0x80 } else { 0x00 } | (frame.opcode as u8);
        header.push(first_byte);

        // Second byte: MASK + payload length
        let masked = frame.mask.is_some();
        let mask_bit = if masked { 0x80 } else { 0x00 };

        let len = frame.payload.len();
        if len < 126 {
            header.push(mask_bit | len as u8);
        } else if len < 65536 {
            header.push(mask_bit | 126);
            header.extend_from_slice(&(len as u16).to_be_bytes());
        } else {
            header.push(mask_bit | 127);
            header.extend_from_slice(&(len as u64).to_be_bytes());
        }

        // Mask key
        if let Some(mask_key) = frame.mask {
            header.extend_from_slice(&mask_key);
        }

        // Write header
        self.stream.write_all(&header).await?;

        // Write payload (masked if needed)
        if let Some(mask_key) = frame.mask {
            let masked_payload: Vec<u8> = frame
                .payload
                .iter()
                .enumerate()
                .map(|(i, &b)| b ^ mask_key[i % 4])
                .collect();
            self.stream.write_all(&masked_payload).await?;
        } else {
            self.stream.write_all(&frame.payload).await?;
        }

        self.stream.flush().await?;
        Ok(())
    }

    /// Check if the connection is closed
    pub fn is_closed(&self) -> bool {
        self.closed
    }
}

/// A WebSocket frame
struct Frame {
    fin: bool,
    opcode: Opcode,
    mask: Option<[u8; 4]>,
    payload: Vec<u8>,
}

/// Simple base64 encoder
fn base64_encode(data: &[u8]) -> String {
    const ALPHABET: &[u8; 64] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let mut result = String::new();
    let mut i = 0;

    while i < data.len() {
        let b0 = data[i] as u32;
        let b1 = if i + 1 < data.len() { data[i + 1] as u32 } else { 0 };
        let b2 = if i + 2 < data.len() { data[i + 2] as u32 } else { 0 };

        let triple = (b0 << 16) | (b1 << 8) | b2;

        result.push(ALPHABET[(triple >> 18 & 0x3F) as usize] as char);
        result.push(ALPHABET[(triple >> 12 & 0x3F) as usize] as char);

        if i + 1 < data.len() {
            result.push(ALPHABET[(triple >> 6 & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }

        if i + 2 < data.len() {
            result.push(ALPHABET[(triple & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }

        i += 3;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_accept_key() {
        // Test vector from RFC 6455
        let key = "dGhlIHNhbXBsZSBub25jZQ==";
        let accept = WebSocket::<tokio::net::TcpStream>::compute_accept_key(key);
        assert_eq!(accept, "s3pPLMBiTxaQ9kYGzzhZRbK+xOo=");
    }

    #[test]
    fn test_base64_encode() {
        assert_eq!(base64_encode(b""), "");
        assert_eq!(base64_encode(b"f"), "Zg==");
        assert_eq!(base64_encode(b"fo"), "Zm8=");
        assert_eq!(base64_encode(b"foo"), "Zm9v");
        assert_eq!(base64_encode(b"foob"), "Zm9vYg==");
        assert_eq!(base64_encode(b"fooba"), "Zm9vYmE=");
        assert_eq!(base64_encode(b"foobar"), "Zm9vYmFy");
    }

    #[test]
    fn test_message_opcode() {
        assert_eq!(Message::Text("hello".into()).opcode(), Opcode::Text);
        assert_eq!(Message::Binary(vec![1, 2, 3]).opcode(), Opcode::Binary);
        assert_eq!(Message::Ping(vec![]).opcode(), Opcode::Ping);
        assert_eq!(Message::Close(None).opcode(), Opcode::Close);
    }
}
