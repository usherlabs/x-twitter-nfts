
use futures::AsyncWriteExt;
use hyper::{body::to_bytes, client::conn::Parts, Body, Request, StatusCode, Method};
use rustls::{Certificate, ClientConfig, RootCertStore};
use serde::{Deserialize, Serialize};
use std::{env, ops::Range, str, sync::Arc};
use tlsn_core::proof::TlsProof;
use tokio::{io::AsyncWriteExt as _, net::TcpStream};
use tokio_rustls::TlsConnector;
use tokio_util::compat::{FuturesAsyncReadCompatExt, TokioAsyncReadCompatExt};
use tracing::debug;
use reqwest;
use std::collections::HashMap;

use tlsn_prover::tls::{Prover, ProverConfig};

// Setting of the notary server — make sure these are the same with those in ../../../notary
const NOTARY_HOST: &str = "127.0.0.1";
const NOTARY_PORT: u16 = 7047;

// Setting of the application server
const SERVER_DOMAIN: &str = "api.twitter.com";

// Configuration of notarization
const NOTARY_MAX_TRANSCRIPT_SIZE: usize = 16384;

/// Response object of the /session API
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotarizationSessionResponse {
    pub session_id: String,
}

/// Request object of the /session API
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotarizationSessionRequest {
    pub client_type: ClientType,
    /// Maximum transcript size in bytes
    pub max_transcript_size: Option<usize>,
}

/// Types of client that the prover is using
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ClientType {
    /// Client that has access to the transport layer
    Tcp,
    /// Client that cannot directly access transport layer, e.g. browser extension
    Websocket,
}

#[tokio::main]
async fn main()  -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    // Load secret variables frome environment for twitter server connection
    dotenv::dotenv().ok();
    let api_key = env::var("API_KEY").unwrap();
    let api_key_secret = env::var("API_KEY_SECRET").unwrap();
    let conversation_id = env::var("CONVERSATION_ID").unwrap();

    // Fetch bearer token
    // as per https://developer.twitter.com/en/docs/authentication/oauth-2-0/bearer-tokens
    let client = reqwest::Client::new();
    let bearer_resp = client.post(format!(
            "https://{SERVER_DOMAIN}/oauth2/token"
        ))
        .basic_auth(api_key, Some(api_key_secret))
        .form(&[("grant_type", "client_credentials")])
        .send().await?.json::<HashMap<String, String>>().await?;

    let access_token = bearer_resp.get("access_token").unwrap();

    debug!("Bearer token obtained");
    // println!("Bearer token {:?}", access_token);

    let (notary_tls_socket, session_id) = setup_notary_connection().await;

    // Basic default prover config using the session_id returned from /session endpoint just now
    let config = ProverConfig::builder()
        .id(session_id)
        .server_dns(SERVER_DOMAIN)
        .build()
        .unwrap();

    // Create a new prover and set up the MPC backend.
    let prover = Prover::new(config)
        .setup(notary_tls_socket.compat())
        .await
        .unwrap();

    let client_socket = tokio::net::TcpStream::connect((SERVER_DOMAIN, 443))
        .await
        .unwrap();

    // Bind the Prover to server connection
    let (tls_connection, prover_fut) = prover.connect(client_socket.compat()).await.unwrap();

    // Spawn the Prover to be run concurrently
    let prover_task = tokio::spawn(prover_fut);

    // Attach the hyper HTTP client to the TLS connection
    let (mut request_sender, connection) = hyper::client::conn::handshake(tls_connection.compat())
        .await
        .unwrap();

    // Spawn the HTTP task to be run concurrently
    let connection_task = tokio::spawn(connection.without_shutdown());

    // Build the HTTP request to fetch the DMs
    let request = Request::builder()
        .uri(format!(
            "https://{SERVER_DOMAIN}/2/tweets?ids={conversation_id}&tweet.fields=created_at&expansions=author_id&user.fields=created_at"
        ))
        .method(Method::GET)
        .header("Authorization", format!("Bearer {access_token}"))
        .header("Host", SERVER_DOMAIN)
        .header("Accept", "*/*")
        .header("Cache-Control", "no-cache")
        .header("Connection", "close")
        // Using "identity" instructs the Server not to use compression for its HTTP response.
        // TLSNotary tooling does not support compression.
        .header("Accept-Encoding", "identity")
        .body(Body::empty())
        .unwrap();

    debug!("Sending request");

    let response = request_sender.send_request(request).await.unwrap();

    debug!("Sent request");

    assert!(response.status() == StatusCode::OK, "{}", response.status());

    debug!("Request OK");

    // Pretty printing :)
    let payload = to_bytes(response.into_body()).await.unwrap().to_vec();
    let parsed =
        serde_json::from_str::<serde_json::Value>(&String::from_utf8_lossy(&payload)).unwrap();
    debug!("{}", serde_json::to_string_pretty(&parsed).unwrap());

    // Close the connection to the server
    let mut client_socket = connection_task.await.unwrap().unwrap().io.into_inner();
    client_socket.close().await.unwrap();

    // The Prover task should be done now, so we can grab it.
    let prover = prover_task.await.unwrap().unwrap();

    // Prepare for notarization
    let mut prover = prover.start_notarize();

    debug!("Start notarize");

    // Identify the ranges in the transcript that contain secrets
    let (public_ranges, private_ranges) =
        find_ranges(prover.sent_transcript().data(), &[access_token.as_bytes()]);

    let recv_len = prover.recv_transcript().data().len();

    let builder = prover.commitment_builder();

    // Collect commitment ids for the outbound transcript
    let mut commitment_ids = public_ranges
        .iter()
        .chain(private_ranges.iter())
        .map(|range| builder.commit_sent(range.clone()).unwrap())
        .collect::<Vec<_>>();

    // Commit to the full received transcript in one shot, as we don't need to redact anything
    commitment_ids.push(builder.commit_recv(0..recv_len).unwrap());

    debug!("Collect commitments {}", commitment_ids.len());

    // Finalize, returning the notarized session
    let notarized_session = prover.finalize().await.unwrap();

    debug!("Notarization complete!");

    // Dump the notarized session to a file
    let mut file = tokio::fs::File::create("twitter_proof.json")
        .await
        .unwrap();
    file.write_all(
        serde_json::to_string_pretty(&notarized_session)
            .unwrap()
            .as_bytes(),
    )
    .await
    .unwrap();

    let session_proof = notarized_session.session_proof();

    let mut proof_builder = notarized_session.data().build_substrings_proof();

    // Reveal everything but the auth token (which was assigned commitment id 2)
    proof_builder.reveal(commitment_ids[0]).unwrap();
    proof_builder.reveal(commitment_ids[1]).unwrap();
    proof_builder.reveal(commitment_ids[3]).unwrap();

    let substrings_proof = proof_builder.build().unwrap();

    let proof = TlsProof {
        session: session_proof,
        substrings: substrings_proof,
    };

    // Dump the proof to a file.
    let mut file = tokio::fs::File::create("twitter_proof.json")
        .await
        .unwrap();
    file.write_all(serde_json::to_string_pretty(&proof).unwrap().as_bytes())
        .await
        .unwrap();

    Ok(())
}

async fn setup_notary_connection() -> (tokio_rustls::client::TlsStream<TcpStream>, String) {
    // Connect to the Notary via TLS-TCP
    let pem_file = str::from_utf8(include_bytes!("../../notary/fixture/tls/rootCA.crt")).unwrap();
    let mut reader = std::io::BufReader::new(pem_file.as_bytes());
    let mut certificates: Vec<Certificate> = rustls_pemfile::certs(&mut reader)
        .unwrap()
        .into_iter()
        .map(Certificate)
        .collect();
    let certificate = certificates.remove(0);

    let mut root_store = RootCertStore::empty();
    root_store.add(&certificate).unwrap();

    let client_notary_config = ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(root_store)
        .with_no_client_auth();
    let notary_connector = TlsConnector::from(Arc::new(client_notary_config));

    let notary_socket = tokio::net::TcpStream::connect((NOTARY_HOST, NOTARY_PORT))
        .await
        .unwrap();

    let notary_tls_socket = notary_connector
        // Require the domain name of notary server to be the same as that in the server cert
        .connect("tlsnotaryserver.io".try_into().unwrap(), notary_socket)
        .await
        .unwrap();

    // Attach the hyper HTTP client to the notary TLS connection to send request to the /session endpoint to configure notarization and obtain session id
    let (mut request_sender, connection) = hyper::client::conn::handshake(notary_tls_socket)
        .await
        .unwrap();

    // Spawn the HTTP task to be run concurrently
    let connection_task = tokio::spawn(connection.without_shutdown());

    // Build the HTTP request to configure notarization
    let payload = serde_json::to_string(&NotarizationSessionRequest {
        client_type: ClientType::Tcp,
        max_transcript_size: Some(NOTARY_MAX_TRANSCRIPT_SIZE),
    })
    .unwrap();

    let request = Request::builder()
        .uri(format!("https://{NOTARY_HOST}:{NOTARY_PORT}/session"))
        .method("POST")
        .header("Host", NOTARY_HOST)
        // Need to specify application/json for axum to parse it as json
        .header("Content-Type", "application/json")
        .body(Body::from(payload))
        .unwrap();

    debug!("Sending configuration request");

    let configuration_response = request_sender.send_request(request).await.unwrap();

    debug!("Sent configuration request");

    assert!(configuration_response.status() == StatusCode::OK);

    debug!("Response OK");

    // Pretty printing :)
    let payload = to_bytes(configuration_response.into_body())
        .await
        .unwrap()
        .to_vec();
    let notarization_response =
        serde_json::from_str::<NotarizationSessionResponse>(&String::from_utf8_lossy(&payload))
            .unwrap();

    debug!("Notarization response: {:?}", notarization_response,);

    // Send notarization request via HTTP, where the underlying TCP connection will be extracted later
    let request = Request::builder()
        // Need to specify the session_id so that notary server knows the right configuration to use
        // as the configuration is set in the previous HTTP call
        .uri(format!(
            "https://{}:{}/notarize?sessionId={}",
            NOTARY_HOST,
            NOTARY_PORT,
            notarization_response.session_id.clone()
        ))
        .method("GET")
        .header("Host", NOTARY_HOST)
        .header("Connection", "Upgrade")
        // Need to specify this upgrade header for server to extract tcp connection later
        .header("Upgrade", "TCP")
        .body(Body::empty())
        .unwrap();

    debug!("Sending notarization request");

    let response = request_sender.send_request(request).await.unwrap();

    debug!("Sent notarization request");

    assert!(response.status() == StatusCode::SWITCHING_PROTOCOLS);

    debug!("Switched protocol OK");

    // Claim back the TLS socket after HTTP exchange is done
    let Parts {
        io: notary_tls_socket,
        ..
    } = connection_task.await.unwrap().unwrap();

    (notary_tls_socket, notarization_response.session_id)
}

/// Find the ranges of the public and private parts of a sequence.
///
/// Returns a tuple of `(public, private)` ranges.
fn find_ranges(seq: &[u8], sub_seq: &[&[u8]]) -> (Vec<Range<usize>>, Vec<Range<usize>>) {
    let mut private_ranges = Vec::new();
    for s in sub_seq {
        // .windows chunks the seq into arrays of length s.len()
        // .enumerate transforms the iterator into a new iterator that pairs (index, element)
        for (idx, w) in seq.windows(s.len()).enumerate() {
            if w == *s {
                // private_ranges resulsts in an array of Ranges - where the range is the start and end of sub_seq[x]
                private_ranges.push(idx..(idx + w.len()));
            }
        }
    }

    let mut sorted_ranges = private_ranges.clone();
    sorted_ranges.sort_by_key(|r| r.start);

    let mut public_ranges = Vec::new();
    let mut last_end = 0;
    // This process here is just omitting the private_ranges
    for r in sorted_ranges {
        if r.start > last_end {
            public_ranges.push(last_end..r.start);
        }
        last_end = r.end;
    }

    // This completes any remaining range
    if last_end < seq.len() {
        public_ranges.push(last_end..seq.len());
    }

    (public_ranges, private_ranges)
}
