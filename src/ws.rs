use anyhow::{Context, Result};
use futures_util::stream::SplitSink;
use futures_util::stream::SplitStream;
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::http::HeaderValue;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

pub type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;
pub type WsSink = SplitSink<WsStream, Message>;
pub type WsSource = SplitStream<WsStream>;

pub async fn connect(url: &str, insecure: bool) -> Result<WsStream> {
    let connector = if insecure {
        use std::sync::Arc;
        let config = rustls::ClientConfig::builder()
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(NoVerify))
            .with_no_client_auth();
        Some(tokio_tungstenite::Connector::Rustls(Arc::new(config)))
    } else {
        None
    };

    let mut request = url.into_client_request()?;
    let headers = request.headers_mut();
    headers.insert(
        "Origin",
        HeaderValue::from_static("https://www.nperf.com"),
    );

    let (ws, _resp) =
        tokio_tungstenite::connect_async_tls_with_config(request, None, false, connector).await?;
    Ok(ws)
}

/// Connect and perform the nperf CONNECT handshake.
pub async fn connect_nperf(url: &str, insecure: bool) -> Result<WsStream> {
    let mut ws = connect(url, insecure).await?;
    ws.send(Message::Text("CONNECT".into())).await?;

    while let Some(msg) = ws.next().await {
        match msg? {
            Message::Text(text) => {
                if text.starts_with("CONNECTED") {
                    return Ok(ws);
                }
                anyhow::bail!("Unexpected response: {}", text);
            }
            _ => continue,
        }
    }
    anyhow::bail!("Connection closed before CONNECTED response")
}

pub async fn connect_nperf_pool(url: &str, n: u32, insecure: bool) -> Result<Vec<WsStream>> {
    let mut handles = Vec::new();
    for _ in 0..n {
        let url = url.to_string();
        handles.push(tokio::spawn(async move {
            connect_nperf(&url, insecure).await
        }));
    }
    let mut streams = Vec::with_capacity(n as usize);
    for h in handles {
        streams.push(h.await?.context("Failed to connect")?);
    }
    Ok(streams)
}

pub fn split(ws: WsStream) -> (WsSink, WsSource) {
    ws.split()
}

/// Certificate verifier that accepts everything (for --insecure).
#[derive(Debug)]
struct NoVerify;

impl rustls::client::danger::ServerCertVerifier for NoVerify {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls_pki_types::CertificateDer<'_>,
        _intermediates: &[rustls_pki_types::CertificateDer<'_>],
        _server_name: &rustls_pki_types::ServerName<'_>,
        _ocsp_response: &[u8],
        _now: rustls_pki_types::UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &rustls_pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &rustls_pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        use rustls::SignatureScheme;
        vec![
            SignatureScheme::RSA_PKCS1_SHA256,
            SignatureScheme::RSA_PKCS1_SHA384,
            SignatureScheme::RSA_PKCS1_SHA512,
            SignatureScheme::ECDSA_NISTP256_SHA256,
            SignatureScheme::ECDSA_NISTP384_SHA384,
            SignatureScheme::RSA_PSS_SHA256,
            SignatureScheme::RSA_PSS_SHA384,
            SignatureScheme::RSA_PSS_SHA512,
            SignatureScheme::ED25519,
        ]
    }
}
