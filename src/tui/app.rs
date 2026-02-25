use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use crossterm::event::{Event, EventStream, KeyCode, KeyModifiers};
use futures_util::{SinkExt, StreamExt};
use ratatui::DefaultTerminal;
use serde::Deserialize;
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;

use crate::client::LimitlessClient;
use crate::tui::ui;
use crate::tui::vwap::{self, VwapResult};

/// Dollar depths for VWAP computation.
pub const VWAP_DEPTHS: &[f64] = &[10.0, 50.0, 100.0, 200.0];

// ── WebSocket message types ──────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WsOrderbookUpdate {
    #[allow(dead_code)]
    market_slug: Option<String>,
    orderbook: WsOrderbook,
}

#[derive(Debug, Deserialize)]
struct WsOrderbook {
    #[serde(default)]
    bids: Vec<WsOrderLevel>,
    #[serde(default)]
    asks: Vec<WsOrderLevel>,
}

#[derive(Debug, Deserialize)]
struct WsOrderLevel {
    price: serde_json::Value, // can be f64 or string
    size: serde_json::Value,  // can be f64 or string
}

impl WsOrderLevel {
    fn price_f64(&self) -> Option<f64> {
        match &self.price {
            serde_json::Value::Number(n) => n.as_f64(),
            serde_json::Value::String(s) => s.parse().ok(),
            _ => None,
        }
    }

    fn size_f64(&self) -> Option<f64> {
        match &self.size {
            serde_json::Value::Number(n) => n.as_f64(),
            serde_json::Value::String(s) => s.parse().ok(),
            _ => None,
        }
    }
}

// ── Messages sent from WS task to main loop ──────────────────────────

pub enum WsMessage {
    OrderbookUpdate {
        bids: Vec<(f64, f64)>,
        asks: Vec<(f64, f64)>,
    },
    Connected,
    Disconnected(String),
    Error(String),
}

// ── App state ────────────────────────────────────────────────────────

pub struct App {
    pub slug: String,
    pub bids: Vec<(f64, f64)>,  // (price, raw_size) — sorted descending
    pub asks: Vec<(f64, f64)>,  // (price, raw_size) — sorted ascending
    pub midpoint: Option<f64>,
    pub spread: Option<f64>,
    pub best_bid: Option<f64>,
    pub best_ask: Option<f64>,
    pub vwaps: Vec<VwapResult>,
    pub last_update: Option<Instant>,
    pub update_count: u64,
    pub connected: bool,
    pub status_msg: String,
    pub should_quit: bool,
}

impl App {
    fn new(slug: String) -> Self {
        Self {
            slug,
            bids: Vec::new(),
            asks: Vec::new(),
            midpoint: None,
            spread: None,
            best_bid: None,
            best_ask: None,
            vwaps: VWAP_DEPTHS
                .iter()
                .map(|&d| VwapResult {
                    depth_usd: d,
                    vwap_buy: None,
                    vwap_sell: None,
                    buy_shares: 0.0,
                    sell_shares: 0.0,
                })
                .collect(),
            last_update: None,
            update_count: 0,
            connected: false,
            status_msg: "Connecting...".to_string(),
            should_quit: false,
        }
    }

    fn update_book(&mut self, bids: Vec<(f64, f64)>, asks: Vec<(f64, f64)>) {
        self.bids = bids;
        self.asks = asks;
        self.last_update = Some(Instant::now());
        self.update_count += 1;

        // Compute derived values
        self.best_bid = self.bids.first().map(|b| b.0);
        self.best_ask = self.asks.first().map(|a| a.0);

        self.midpoint = match (self.best_bid, self.best_ask) {
            (Some(bid), Some(ask)) => Some((bid + ask) / 2.0),
            _ => None,
        };

        self.spread = match (self.best_bid, self.best_ask) {
            (Some(bid), Some(ask)) => Some(ask - bid),
            _ => None,
        };

        // Recompute VWAPs
        self.vwaps = vwap::compute_vwaps(&self.bids, &self.asks, VWAP_DEPTHS);
    }

    /// Populate from REST OrderbookResponse on startup.
    fn load_from_rest(&mut self, book: &crate::client::trading::OrderbookResponse) {
        let bids: Vec<(f64, f64)> = book
            .bids
            .iter()
            .map(|l| {
                let size_f64: f64 = l.size.to_string().parse().unwrap_or(0.0);
                (l.price, size_f64)
            })
            .collect();
        let asks: Vec<(f64, f64)> = book
            .asks
            .iter()
            .map(|l| {
                let size_f64: f64 = l.size.to_string().parse().unwrap_or(0.0);
                (l.price, size_f64)
            })
            .collect();
        self.update_book(bids, asks);
        self.status_msg = "Loaded via REST, connecting WS...".to_string();
    }
}

// ── Main entry point ─────────────────────────────────────────────────

pub async fn run_monitor(slug: &str, api_key: Option<&str>) -> Result<()> {
    let mut app = App::new(slug.to_string());

    // Fetch initial orderbook via REST for instant display
    let client = LimitlessClient::new(api_key)?;
    match client.get_orderbook(slug).await {
        Ok(book) => app.load_from_rest(&book),
        Err(e) => {
            app.status_msg = format!("REST fetch failed: {}", e);
        }
    }

    // Set up terminal
    let mut terminal = ratatui::init();
    terminal.clear()?;

    let result = run_event_loop(&mut terminal, &mut app).await;

    // Restore terminal
    ratatui::restore();

    result
}

async fn run_event_loop(terminal: &mut DefaultTerminal, app: &mut App) -> Result<()> {
    let (tx, mut rx) = mpsc::unbounded_channel::<WsMessage>();

    // Spawn WebSocket task
    let slug_owned = app.slug.clone();
    let ws_handle = tokio::spawn(async move {
        if let Err(e) = ws_connect_and_subscribe(&slug_owned, tx.clone()).await {
            let _ = tx.send(WsMessage::Error(format!("{:#}", e)));
        }
    });

    let mut event_stream = EventStream::new();
    let mut tick_interval = tokio::time::interval(Duration::from_millis(250));

    // Initial draw
    terminal.draw(|f| ui::render(f, app))?;

    loop {
        tokio::select! {
            // Terminal key events
            maybe_event = event_stream.next() => {
                if let Some(Ok(event)) = maybe_event {
                    if let Event::Key(key) = event {
                        match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => {
                                app.should_quit = true;
                            }
                            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                app.should_quit = true;
                            }
                            _ => {}
                        }
                    }
                }
            }
            // WebSocket messages
            Some(msg) = rx.recv() => {
                match msg {
                    WsMessage::OrderbookUpdate { bids, asks } => {
                        app.update_book(bids, asks);
                    }
                    WsMessage::Connected => {
                        app.connected = true;
                        app.status_msg = "Connected".to_string();
                    }
                    WsMessage::Disconnected(reason) => {
                        app.connected = false;
                        app.status_msg = format!("Disconnected: {}", reason);
                    }
                    WsMessage::Error(e) => {
                        app.status_msg = format!("Error: {}", e);
                    }
                }
            }
            // Tick for redraw
            _ = tick_interval.tick() => {}
        }

        if app.should_quit {
            break;
        }

        terminal.draw(|f| ui::render(f, app))?;
    }

    // Clean up WS task
    ws_handle.abort();
    let _ = ws_handle.await;

    Ok(())
}

// ── Manual Socket.IO v4 over WebSocket ───────────────────────────────
//
// Socket.IO v4 uses Engine.IO v4 as transport. The wire protocol:
//
// Engine.IO packet types: 0=open, 1=close, 2=ping, 3=pong, 4=message
// Socket.IO packet types (inside EIO message): 0=connect, 1=disconnect,
//   2=event, 3=ack, 4=connect_error, 5=binary_event, 6=binary_ack
//
// Full packet = "{eio_type}" or "{eio_type}{sio_type}/namespace,{json_data}"
//
// Examples:
//   EIO open:        0{"sid":"abc","pingInterval":25000,"pingTimeout":20000}
//   SIO connect:     40/markets,           (we send)
//   SIO connect ack: 40/markets,{"sid":"x"} (server responds)
//   SIO event:       42/markets,["eventName",{data}]
//   EIO ping:        2                     (server sends)
//   EIO pong:        3                     (we respond)

const NAMESPACE: &str = "/markets";

async fn ws_connect_and_subscribe(
    slug: &str,
    tx: mpsc::UnboundedSender<WsMessage>,
) -> Result<()> {
    // Socket.IO connects via Engine.IO endpoint
    let ws_url = format!(
        "{}/socket.io/?EIO=4&transport=websocket",
        crate::constants::WS_URL
    );

    let (ws_stream, _response) = tokio_tungstenite::connect_async(&ws_url)
        .await
        .context("Failed to connect WebSocket")?;

    let (mut write, mut read) = ws_stream.split();

    // Step 1: Wait for Engine.IO OPEN packet (type 0)
    let open_msg = read
        .next()
        .await
        .ok_or_else(|| anyhow::anyhow!("WS closed before EIO open"))?
        .context("Error reading EIO open")?;

    let open_text = open_msg
        .into_text()
        .context("EIO open not text")?;

    if !open_text.starts_with('0') {
        anyhow::bail!("Expected EIO open (0...), got: {}", &open_text[..open_text.len().min(80)]);
    }

    // Step 2: Send Socket.IO CONNECT to /markets namespace
    // Format: "40/markets,"
    let connect_packet = format!("40{},", NAMESPACE);
    write
        .send(Message::Text(connect_packet.into()))
        .await
        .context("Failed to send SIO connect")?;

    // Step 3: Wait for Socket.IO CONNECT ACK (40/markets,{...})
    let ack_msg = read
        .next()
        .await
        .ok_or_else(|| anyhow::anyhow!("WS closed before SIO connect ack"))?
        .context("Error reading SIO connect ack")?;

    let ack_text = ack_msg.into_text().unwrap_or_default();
    let expected_prefix = format!("40{},", NAMESPACE);
    if !ack_text.starts_with(&expected_prefix) {
        // Could be a connect_error (44/markets,{...})
        if ack_text.starts_with(&format!("44{},", NAMESPACE)) {
            anyhow::bail!("Socket.IO connect error: {}", &ack_text);
        }
        anyhow::bail!(
            "Expected SIO connect ack (40/markets,...), got: {}",
            &ack_text[..ack_text.len().min(80)]
        );
    }

    let _ = tx.send(WsMessage::Connected);

    // Step 4: Subscribe to market prices
    let subscribe_event = serde_json::json!(["subscribe_market_prices", {"marketSlugs": [slug]}]);
    let subscribe_packet = format!("42{},{}", NAMESPACE, subscribe_event);
    write
        .send(Message::Text(subscribe_packet.into()))
        .await
        .context("Failed to send subscribe")?;

    // Step 5: Read loop — handle pings, events, and disconnects
    while let Some(msg_result) = read.next().await {
        let msg = match msg_result {
            Ok(m) => m,
            Err(e) => {
                let _ = tx.send(WsMessage::Disconnected(format!("WS error: {}", e)));
                break;
            }
        };

        match msg {
            Message::Text(text) => {
                let text = text.to_string();
                if text == "2" {
                    // Engine.IO PING → respond with PONG
                    if write.send(Message::Text("3".into())).await.is_err() {
                        break;
                    }
                } else if text == "3" {
                    // Engine.IO PONG — ignore
                } else if text.starts_with("42") {
                    // Socket.IO EVENT
                    if let Some(parsed) = parse_sio_event(&text) {
                        let _ = tx.send(parsed);
                    }
                } else if text.starts_with("41") {
                    // Socket.IO DISCONNECT
                    let _ = tx.send(WsMessage::Disconnected("server sent disconnect".to_string()));
                    break;
                }
                // Ignore other packet types (acks, binary, etc.)
            }
            Message::Ping(data) => {
                // WebSocket-level ping (separate from Engine.IO ping)
                if write.send(Message::Pong(data)).await.is_err() {
                    break;
                }
            }
            Message::Close(_) => {
                let _ = tx.send(WsMessage::Disconnected("WS closed".to_string()));
                break;
            }
            _ => {}
        }
    }

    let _ = tx.send(WsMessage::Disconnected("WS stream ended".to_string()));
    Ok(())
}

/// Parse a Socket.IO v4 EVENT packet.
///
/// Format: `42/markets,["eventName", {data}]`
///
/// We only care about `orderbookUpdate` events.
fn parse_sio_event(raw: &str) -> Option<WsMessage> {
    // Strip "42" prefix
    let after_type = raw.strip_prefix("42")?;

    // Strip namespace: "/markets,"
    let json_str = if after_type.starts_with(NAMESPACE) {
        let after_ns = &after_type[NAMESPACE.len()..];
        after_ns.strip_prefix(',')?
    } else {
        // No namespace prefix — default namespace
        after_type
    };

    // Parse as JSON array: ["eventName", {data}]
    let arr: Vec<serde_json::Value> = serde_json::from_str(json_str).ok()?;
    if arr.len() < 2 {
        return None;
    }

    let event_name = arr[0].as_str()?;
    if event_name != "orderbookUpdate" {
        return None;
    }

    let data = &arr[1];
    let update: WsOrderbookUpdate = serde_json::from_value(data.clone()).ok()?;

    let bids: Vec<(f64, f64)> = update
        .orderbook
        .bids
        .iter()
        .filter_map(|l| Some((l.price_f64()?, l.size_f64()?)))
        .collect();

    let asks: Vec<(f64, f64)> = update
        .orderbook
        .asks
        .iter()
        .filter_map(|l| Some((l.price_f64()?, l.size_f64()?)))
        .collect();

    Some(WsMessage::OrderbookUpdate { bids, asks })
}
