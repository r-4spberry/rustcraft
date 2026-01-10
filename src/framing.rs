use bytes::BytesMut;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};

use crate::packet::Packet;
use crate::protocol::try_read_varint_len;

pub async fn read_packet(r: &mut OwnedReadHalf) -> anyhow::Result<Option<Packet>> {
    let mut buf = BytesMut::with_capacity(1024);
    r.readable().await?;
    let mut len: usize;
    let mut header_len: usize = 0;
    let mut packet_len: usize = 0;
    loop {
        len = r.read_buf(&mut buf).await?;

        if len == 0 {
            return Ok(None);
        }
        if let Some(ass) = try_read_varint_len(&buf)? {
            (packet_len, header_len) = ass;
        }
        if buf.len() < packet_len + header_len {
            buf.reserve(packet_len + header_len - buf.len());
        }

        if buf.len() == packet_len + header_len {
            break;
        }

        assert!(buf.len() < packet_len + header_len)
    }
    buf.split_to(header_len);

    Ok(Some(Packet::new(buf.into())))
}
