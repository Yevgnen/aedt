//! Speech: what the reader says, written down by the provider's speech-to-text.

use nsh_core::{platform::HttpRequest, prelude::*};
use serde::Deserialize;

use crate::models::failure;

/// The boundary a multipart body is cut with: one request per recording, and
/// nothing of the app's own is in the audio, so one fixed name is enough.
const BOUNDARY: &str = "aedt-voice-boundary";

/// The transcript of `wav`, a 16-bit PCM WAV file, by the provider at
/// `base_url` with `api_key`. The provider hears the language itself: the
/// request asks for none.
pub async fn transcribe(
    base_url: &str,
    api_key: &str,
    wav: Vec<u8>,
) -> Result<String, SharedString> {
    let url = format!("{}/v1/tools/asr", root(base_url));
    let request = HttpRequest::post(url, multipart(&wav))
        .header("Authorization", format!("Bearer {api_key}"))
        .header(
            "Content-Type",
            format!("multipart/form-data; boundary={BOUNDARY}"),
        );
    let response = fetch(request).await.map_err(SharedString::from)?;
    let body = response.text();
    if !response.ok() {
        return Err(failure(response.status(), &body).into());
    }

    let transcript: Transcript = serde_json::from_str(&body)
        .map_err(|error| SharedString::from(format!("Couldn't read the transcript: {error}")))?;

    Ok(transcript.text.trim().to_owned())
}

/// The provider's own root in `base_url`: the settings may keep its base URL
/// with the version path requests hang under, and the speech path carries its
/// own.
fn root(base_url: &str) -> &str {
    let base = base_url.trim().trim_end_matches('/');
    base.strip_suffix("/v1")
        .or_else(|| base.strip_suffix("/beta"))
        .unwrap_or(base)
}

/// `wav` as the one file of a multipart body.
fn multipart(wav: &[u8]) -> Vec<u8> {
    let mut body = Vec::with_capacity(wav.len() + 256);
    body.extend_from_slice(
        format!(
            "--{BOUNDARY}\r\n\
             Content-Disposition: form-data; name=\"file\"; filename=\"speech.wav\"\r\n\
             Content-Type: audio/wav\r\n\r\n"
        )
        .as_bytes(),
    );
    body.extend_from_slice(wav);
    body.extend_from_slice(format!("\r\n--{BOUNDARY}--\r\n").as_bytes());
    body
}

/// A provider's speech-to-text answer, of what the app reads.
#[derive(Deserialize)]
struct Transcript {
    text: String,
}
