// === FINAL ENHANCEMENT: Multiplayer TCP Implementation + Full Game Test + README with Screenshots ===

use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};

/// Default port used for multiplayer connections.
pub const DEFAULT_PORT: u16 = 8080;

/// Selects the networking mode for the current session.
#[derive(Debug, Clone)]
pub enum NetworkMode {
    /// Standard local single-player game (default).
    SinglePlayer,
    /// Host a multiplayer game on the given port.
    Host(u16),
    /// Connect to a hosted multiplayer game at the given address and port.
    Client(String, u16),
}

/// Wire message exchanged between host and client each turn (newline-delimited JSON).
#[derive(Debug, Clone)]
pub struct NetMessage {
    pub turn: u32,
    /// Zone chosen by the local human player this turn (e.g. "5").
    pub move_zone: String,
    /// Zone guessed by the local player for the opponent's move (if defending).
    pub guess_zone: Option<String>,
}

impl NetMessage {
    /// Serialize to a single JSON line (no external crate needed).
    pub fn to_json_line(&self) -> String {
        let guess = match &self.guess_zone {
            Some(g) => format!("\"{}\"", g),
            None => "null".to_string(),
        };
        format!(
            "{{\"turn\":{},\"move_zone\":\"{}\",\"guess_zone\":{}}}\n",
            self.turn, self.move_zone, guess
        )
    }

    /// Parse from a single JSON line produced by `to_json_line`.
    pub fn from_json_line(line: &str) -> Option<Self> {
        // Minimal hand-rolled parser — avoids serde dependency for this one struct.
        let turn = extract_u32(line, "\"turn\":")?;
        let move_zone = extract_str(line, "\"move_zone\":\"")?;
        let guess_zone = extract_optional_str(line, "\"guess_zone\":");
        Some(NetMessage { turn, move_zone, guess_zone })
    }
}

// ---------------------------------------------------------------------------
// Tiny JSON field extractors (avoids a full serde dependency for wire messages)
// ---------------------------------------------------------------------------

fn extract_u32(s: &str, key: &str) -> Option<u32> {
    let start = s.find(key)? + key.len();
    let rest = &s[start..];
    let end = rest.find(|c: char| !c.is_ascii_digit()).unwrap_or(rest.len());
    rest[..end].parse().ok()
}

fn extract_str(s: &str, key: &str) -> Option<String> {
    let start = s.find(key)? + key.len();
    let rest = &s[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

fn extract_optional_str(s: &str, key: &str) -> Option<String> {
    let start = s.find(key)? + key.len();
    let rest = s[start..].trim_start();
    if rest.starts_with("null") {
        return None;
    }
    // Should be "value"
    let inner = rest.trim_start_matches('"');
    let end = inner.find('"')?;
    Some(inner[..end].to_string())
}

// ---------------------------------------------------------------------------
// Multiplayer session handles
// ---------------------------------------------------------------------------

/// Established multiplayer connection — wraps the TCP stream for easy send/recv.
pub struct MultiplayerSession {
    stream: TcpStream,
    reader: BufReader<TcpStream>,
}

impl MultiplayerSession {
    /// Send a NetMessage to the remote peer.
    pub fn send(&mut self, msg: &NetMessage) -> std::io::Result<()> {
        self.stream.write_all(msg.to_json_line().as_bytes())
    }

    /// Receive a NetMessage from the remote peer (blocks until line arrives or peer closes).
    pub fn recv(&mut self) -> Option<NetMessage> {
        let mut line = String::new();
        match self.reader.read_line(&mut line) {
            Ok(0) => None,   // connection closed
            Ok(_) => NetMessage::from_json_line(line.trim()),
            Err(_) => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Public API used by main.rs
// ---------------------------------------------------------------------------

/// Start hosting: bind on `port`, wait for exactly one client, exchange the shared
/// RNG seed, and return (session, seed). Prints progress to stdout.
pub fn host_game(port: u16) -> std::io::Result<(MultiplayerSession, u64)> {
    let addr = format!("0.0.0.0:{}", port);
    println!("🌐 Hosting on {} — waiting for opponent...", addr);
    let listener = TcpListener::bind(&addr)?;
    let (stream, peer) = listener.accept()?;
    println!("✅ Opponent connected from {}", peer);

    // Generate a seed and send it to the client.
    use std::time::{SystemTime, UNIX_EPOCH};
    let seed: u64 = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(42);

    let seed_line = format!("{{\"seed\":{}}}\n", seed);
    let mut stream_clone = stream.try_clone()?;
    stream_clone.write_all(seed_line.as_bytes())?;

    let session = MultiplayerSession {
        reader: BufReader::new(stream.try_clone()?),
        stream,
    };
    Ok((session, seed))
}

/// Join a hosted game: connect to `addr:port`, receive the shared RNG seed, and
/// return (session, seed).
pub fn join_game(addr: &str, port: u16) -> std::io::Result<(MultiplayerSession, u64)> {
    let full_addr = format!("{}:{}", addr, port);
    println!("🌐 Connecting to {}...", full_addr);
    let stream = TcpStream::connect(&full_addr)?;
    println!("✅ Connected to host");

    let mut reader = BufReader::new(stream.try_clone()?);
    let mut seed_line = String::new();
    reader.read_line(&mut seed_line)?;
    let seed = extract_u32(&seed_line, "\"seed\":").unwrap_or(42) as u64;

    let session = MultiplayerSession {
        reader,
        stream,
    };
    Ok((session, seed))
}

/// Pretty-print a disconnect warning. Called when `recv` returns `None`.
pub fn on_disconnect() {
    eprintln!("\n⚠️  Opponent disconnected. Ending game.");
}
