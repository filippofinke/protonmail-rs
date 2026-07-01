//! Attachments API: binary download + multipart upload.

use crate::error::Result;
use crate::transport::{Doer, Request};
use serde::Deserialize;

const BOUNDARY: &str = "----protoncliBOUNDARYx7MA4YWxkTrZu0gW";

#[derive(Deserialize)]
struct UploadResp {
    #[serde(rename = "Attachment")]
    attachment: AttachmentId,
}
#[derive(Deserialize)]
struct AttachmentId {
    #[serde(rename = "ID")]
    id: String,
}

enum Part<'a> {
    Text(&'a str, &'a str),
    File(&'a str, &'a [u8]),
}

fn build_multipart(parts: &[Part]) -> Vec<u8> {
    let mut out = Vec::new();
    for p in parts {
        out.extend_from_slice(format!("--{BOUNDARY}\r\n").as_bytes());
        match p {
            Part::Text(name, value) => {
                out.extend_from_slice(
                    format!("Content-Disposition: form-data; name=\"{name}\"\r\n\r\n").as_bytes(),
                );
                out.extend_from_slice(value.as_bytes());
            }
            Part::File(name, data) => {
                out.extend_from_slice(
                    format!(
                        "Content-Disposition: form-data; name=\"{name}\"; filename=\"blob\"\r\n\
                         Content-Type: application/octet-stream\r\n\r\n"
                    )
                    .as_bytes(),
                );
                out.extend_from_slice(data);
            }
        }
        out.extend_from_slice(b"\r\n");
    }
    out.extend_from_slice(format!("--{BOUNDARY}--\r\n").as_bytes());
    out
}

/// Download the raw (encrypted) attachment data packet.
pub async fn get_attachment<D: Doer>(d: &D, id: &str) -> Result<Vec<u8>> {
    let resp = d
        .do_raw(Request::get(format!("/mail/v4/attachments/{id}")))
        .await?;
    Ok(resp.body)
}

/// Upload an encrypted attachment to a draft. Returns the new attachment ID.
#[allow(clippy::too_many_arguments)]
pub async fn upload_attachment<D: Doer>(
    d: &D,
    filename: &str,
    message_id: &str,
    content_id: &str,
    mime_type: &str,
    key_packets: &[u8],
    data_packet: &[u8],
    signature: &[u8],
) -> Result<String> {
    let body = build_multipart(&[
        Part::Text("Filename", filename),
        Part::Text("MessageID", message_id),
        Part::Text("ContentID", content_id),
        Part::Text("MIMEType", mime_type),
        Part::File("KeyPackets", key_packets),
        Part::File("DataPacket", data_packet),
        Part::File("Signature", signature),
    ]);
    let content_type = format!("multipart/form-data; boundary={BOUNDARY}");
    let r: UploadResp = d
        .decode(Request::post("/mail/v4/attachments").raw(body, content_type))
        .await?;
    Ok(r.attachment.id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn multipart_contains_all_parts() {
        let body = build_multipart(&[
            Part::Text("Filename", "f.txt"),
            Part::File("DataPacket", b"\x00\x01"),
        ]);
        let s = String::from_utf8_lossy(&body);
        assert!(s.contains("name=\"Filename\""));
        assert!(s.contains("f.txt"));
        assert!(s.contains("name=\"DataPacket\""));
        assert!(s.contains("application/octet-stream"));
        assert!(s.trim_end().ends_with(&format!("--{BOUNDARY}--")));
    }
}
