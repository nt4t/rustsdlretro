//! # API Module — Shared State for WebSocket Control
//!
//! Provides thread-safe shared state that bridges the WebSocket server
//! and the main emulation loop. All state is behind `#[cfg(feature = "api")]`.

use std::cell::Cell;
use std::sync::{Arc, Mutex};
use crate::ResolutionState;

// ─── Input Types ──────────────────────────────────────────────────────────────

/// Button names accepted from the WebSocket protocol.
/// Maps directly to libretro RETRO_DEVICE_ID_JOYPAD values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Button {
    B, A, Y, X, L, R, Start, Select, Up, Down, Left, Right,
}

/// Joypad state snapshot for a single port from the web client.
#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct InputSnapshot {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub b: bool,
    pub a: bool,
    pub y: bool,
    pub x: bool,
    pub l: bool,
    pub r: bool,
    pub start: bool,
    pub select: bool,
}

impl InputSnapshot {
    /// Convert to a bitmask compatible with libretro `retro_input_state_t`.
    #[inline]
    pub fn get_button(&self, id: u32) -> i16 {
        match id {
            0 => self.b as i16,           // RETRO_DEVICE_ID_JOYPAD_B
            1 => self.y as i16,           // RETRO_DEVICE_ID_JOYPAD_Y
            2 => self.select as i16,      // RETRO_DEVICE_ID_JOYPAD_SELECT
            3 => self.start as i16,       // RETRO_DEVICE_ID_JOYPAD_START
            4 => self.up as i16,          // RETRO_DEVICE_ID_JOYPAD_UP
            5 => self.down as i16,        // RETRO_DEVICE_ID_JOYPAD_DOWN
            6 => self.left as i16,        // RETRO_DEVICE_ID_JOYPAD_LEFT
            7 => self.right as i16,       // RETRO_DEVICE_ID_JOYPAD_RIGHT
            8 => self.a as i16,           // RETRO_DEVICE_ID_JOYPAD_A
            9 => self.x as i16,           // RETRO_DEVICE_ID_JOYPAD_X
            10 => self.l as i16,          // RETRO_DEVICE_ID_JOYPAD_L
            11 => self.r as i16,          // RETRO_DEVICE_ID_JOYPAD_R
            _ => -1,                      // Unknown button — not pressed
        }
    }
}

/// Combined input state for all player ports.
#[derive(Debug, Clone, Default)]
pub struct AllInputs {
    pub port0: InputSnapshot,
    pub port1: InputSnapshot,
    pub port2: InputSnapshot,
    pub port3: InputSnapshot,
}

impl AllInputs {
    #[inline]
    pub fn get(&self, port: u32) -> &InputSnapshot {
        match port {
            0 => &self.port0,
            1 => &self.port1,
            2 => &self.port2,
            3 => &self.port3,
            _ => &self.port0,
        }
    }

    #[inline]
    pub fn get_mut(&mut self, port: u32) -> &mut InputSnapshot {
        match port {
            0 => &mut self.port0,
            1 => &mut self.port1,
            2 => &mut self.port2,
            3 => &mut self.port3,
            _ => &mut self.port0,
        }
    }

    #[inline]
    pub fn get_state(&self, port: u32, id: u32) -> i16 {
        let snapshots = match port {
            0 => &self.port0,
            1 => &self.port1,
            2 => &self.port2,
            3 => &self.port3,
            _ => &self.port0,
        };
        snapshots.get_button(id)
    }
}

// ─── Protocol Messages ────────────────────────────────────────────────────────

/// WebSocket protocol message types from client to server.
#[derive(Debug, serde::Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    /// Joypad state snapshot (sent continuously or on change)
    Input { port: u32, buttons: InputSnapshot },
    /// Run exactly one frame (pauses after)
    Step,
    /// Resume continuous playback
    Play,
    /// Pause emulation
    Pause,
    /// Trigger save state (same as F2)
    SaveState,
    /// Trigger load state (same as F4)
    LoadState,
    /// Set core option dynamically
    SetOption { key: String, value: String },
}

/// WebSocket protocol message types from server to client.
#[derive(Debug, serde::Serialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    /// Status update (running state, current FPS, resolution)
    Status { running: bool, fps: f64, width: u32, height: u32 },
    /// Acknowledge single frame executed (for step mode)
    FrameDone,
    /// Flash message relay (e.g., "State Saved")
    Flash { message: String, duration_ms: u64 },
    /// Error notification
    Error { message: String },
}

// ─── Shared State ─────────────────────────────────────────────────────────────

/// Thread-safe shared state between the WebSocket server and main emulation loop.
pub type SharedApiState = Arc<Mutex<ApiState>>;

/// A captured frame ready for encoding (RGBA pixels).
#[derive(Clone)]
pub struct CapturedFrame {
    /// Width in pixels
    pub width: u32,
    /// Height in pixels  
    pub height: u32,
    /// RGBA pixel data (4 bytes per pixel)
    pub pixels: Vec<u8>,
}

impl CapturedFrame {
    /// Create a new empty frame buffer with the given dimensions.
    pub fn new(width: u32, height: u32) -> Self {
        let size = (width * height * 4) as usize;
        Self { width, height, pixels: vec![0u8; size] }
    }
}

/// Inner state that can be shared across threads via Mutex<Arc>.
#[derive(Default)]
pub struct ApiState {
    /// Whether to run continuous playback (false = paused/idle).
    pub running: bool,
    /// Requested state: pause emulation.
    pub paused: bool,
    /// Trigger exactly one frame of execution, then return to idle.
    step_frame: bool,
    /// One-shot request to save a state snapshot.
    save_requested: bool,
    /// One-shot request to load a state snapshot.
    load_requested: bool,
    /// Current input states from the web client (4 ports).
    inputs: AllInputs,
    /// Core option changes pending application.
    options_pending: Vec<(String, String)>,
    /// Shared reference to the core's ResolutionState for status updates.
    resolution: Option<Arc<Mutex<ResolutionState>>>,
    fps: f64,
    /// Latest known resolution width.
    width: u32,
    /// Latest known resolution height.
    height: u32,

    // ── Frame Streaming State ────────────────────────────────────────────
    /// Channel sender for frame snapshots (main thread → server).
    #[cfg(all(feature = "tokio", feature = "tungstenite"))]
    pub frame_tx: Option<crossbeam_channel::Sender<CapturedFrame>>,
}

impl ApiState {
    pub fn new() -> Self {
        Self { running: true, ..Self::default() }
    }

    // ── Frame control ───────────────────────────────────────────────────────

    /// Check and consume the frame-step flag. Returns true if a step was requested.
    #[inline]
    pub fn consume_frame_step(&mut self) -> bool {
        std::mem::replace(&mut self.step_frame, false)
    }

    /// Set the frame-step flag for single-frame execution.
    #[inline]
    pub fn request_frame_step(&mut self) {
        self.step_frame = true;
        self.paused = true;
    }

    // ── Save/Load state ─────────────────────────────────────────────────────

    /// Check and consume the save-state flag. Returns true if a save was requested.
    #[inline]
    pub fn take_save_request(&mut self) -> bool {
        std::mem::replace(&mut self.save_requested, false)
    }

    /// Trigger a save state request from the API.
    #[inline]
    pub fn request_save_state(&mut self) {
        self.save_requested = true;
    }

    /// Check and consume the load-state flag. Returns true if a load was requested.
    #[inline]
    pub fn take_load_request(&mut self) -> bool {
        std::mem::replace(&mut self.load_requested, false)
    }

    /// Trigger a load state request from the API.
    #[inline]
    pub fn request_load_state(&mut self) {
        self.load_requested = true;
    }

    // ── Playback control ────────────────────────────────────────────────────

    /// Start continuous playback.
    #[inline]
    pub fn start_playback(&mut self) {
        self.running = true;
        self.paused = false;
    }

    /// Pause emulation.
    #[inline]
    pub fn pause_emulation(&mut self) {
        self.running = false;
        self.paused = true;
    }

    // ── Input ───────────────────────────────────────────────────────────────

    /// Get input state for a given port and libretro button ID.
    #[inline]
    pub fn get_input_state(&mut self, port: u32, id: u32) -> i16 {
        self.inputs.get_state(port, id)
    }

    // ── Options ─────────────────────────────────────────────────────────────

    /// Get all pending option changes. Consumes the queue.
    pub fn take_pending_options(&mut self) -> Vec<(String, String)> {
        std::mem::take(&mut self.options_pending)
    }

    /// Queue a core option change for application in the main loop.
    pub fn queue_option_change(&mut self, key: String, value: String) {
        // Remove any existing entry with the same key first
        self.options_pending.retain(|(k, _)| k != &key);
        self.options_pending.push((key, value));
    }

    // ── Resolution / Status ─────────────────────────────────────────────────

    /// Update resolution and FPS from the emulation loop.
    pub fn update_resolution(&mut self, fps: f64, width: u32, height: u32) {
        self.fps = fps;
        self.width = width;
        self.height = height;
    }

    /// Get current status snapshot for sending to clients.
    pub fn get_status(&self) -> (bool, f64, u32, u32) {
        (self.running, self.fps, self.width, self.height)
    }

    /// Whether emulation should run this frame.
    #[inline]
    pub fn is_running(&self) -> bool {
        self.running && !self.paused
    }

    /// Set the shared resolution state reference (called from main loop after ROM load).
    pub fn set_resolution_source(&mut self, res: Arc<Mutex<ResolutionState>>) {
        self.resolution = Some(res);
    }

    /// Refresh FPS and resolution from the shared ResolutionState.
    pub fn refresh_status_from_res(&mut self) {
        if let Some(ref res_arc) = self.resolution {
            if let Ok(res) = res_arc.lock() {
                self.fps = res.fps;
                self.width = res.width;
                self.height = res.height;
            }
        }
    }

    /// Get input snapshot for a port (used by main loop to merge into InputReader).
    pub fn get_input_snapshot(&self, port: u32) -> &InputSnapshot {
        self.inputs.get(port)
    }

    // ── Frame Streaming ─────────────────────────────────────────────────────

    /// Push a captured frame into the streaming channel.
    #[cfg(all(feature = "tokio", feature = "tungstenite"))]
    pub fn push_frame(&self, frame: CapturedFrame) {
        if let Some(ref tx) = self.frame_tx {
            // Non-blocking send — drop frames if client is slow (prevents blocking emulation)
            let _ = tx.try_send(frame);
        }
    }

    /// Get the frame channel sender for video backends to push snapshots.
    #[cfg(all(feature = "tokio", feature = "tungstenite"))]
    pub fn get_frame_tx(&self) -> Option<crossbeam_channel::Sender<CapturedFrame>> {
        self.frame_tx.clone()
    }
}

/// Create a new API state and spawn the WebSocket server on a background thread.
pub fn create_api_state(resolution: Arc<Mutex<ResolutionState>>) -> SharedApiState {
    let mut inner = ApiState::new();
    inner.set_resolution_source(Arc::clone(&resolution));
    
    // Create frame streaming channel (bounded to prevent memory issues)
    #[cfg(all(feature = "tokio", feature = "tungstenite"))]
    {
        let (frame_tx, frame_rx) = crossbeam_channel::bounded::<CapturedFrame>(4);
        inner.frame_tx = Some(frame_tx);
        // Store receiver for server to read from
        *FRAME_RX.lock().unwrap() = Some(frame_rx);
    }

    let inner = Arc::new(Mutex::new(inner));

    // Spawn the server thread (requires both tokio and tungstenite)
    #[cfg(all(feature = "tokio", feature = "tungstenite"))]
    {
        use std::net::SocketAddr;
        let addr: SocketAddr = ([0, 0, 0, 0], 18932).into();

        eprintln!("[API] Spawning WebSocket server on {}", addr);
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to build tokio runtime for API server");

        // Clone inner for the spawned thread before moving it into closure
        let state_for_server = Arc::clone(&inner);
        std::thread::spawn(move || {
            eprintln!("[API] Tokio runtime started, entering event loop...");
            if let Err(e) = rt.block_on(api_server::run(addr, state_for_server)) {
                eprintln!("[API] Server error: {}", e);
            }
        });
    }

    #[cfg(not(feature = "tokio"))]
    {
        // Fallback: no server started — API state available but not accessible remotely
        eprintln!("[API] Feature 'api' enabled but tokio runtime unavailable; server not started");
    }

    inner
}

/// Global frame sender for video backends to push snapshots.
#[cfg(all(feature = "tokio", feature = "tungstenite"))]
pub static FRAME_TX: std::sync::Mutex<Option<crossbeam_channel::Sender<CapturedFrame>>> = 
    std::sync::Mutex::new(None);

/// Global frame receiver for the server to distribute to clients.
#[cfg(all(feature = "tokio", feature = "tungstenite"))]
pub static FRAME_RX: std::sync::Mutex<Option<crossbeam_channel::Receiver<CapturedFrame>>> = 
    std::sync::Mutex::new(None);

/// Initialize the global frame channel for streaming. Called once from main.rs after API state creation.
#[cfg(all(feature = "tokio", feature = "tungstenite"))]
pub fn init_frame_streaming(api_state: &SharedApiState) {
    if let Some(tx) = api_state.lock().unwrap().get_frame_tx() {
        *FRAME_TX.lock().unwrap() = Some(tx);
        // Also store the receiver (which is stored in ApiState.frame_rx internally)
        eprintln!("[API] Frame streaming channel initialized");
    }
}

/// Push a captured frame into the streaming channel. Called from video backends.
#[cfg(all(feature = "tokio", feature = "tungstenite"))]
pub fn push_captured_frame(frame: CapturedFrame) {
    if let Some(ref tx) = *FRAME_TX.lock().unwrap() {
        // Non-blocking send — drop frames if channel is full (prevents blocking emulation)
        let _ = tx.try_send(frame);
    }
}

// ─── WebSocket Server (requires tokio + tungstenite) ──────────────────────────

#[cfg(all(feature = "tokio", feature = "tungstenite"))]
pub mod api_server {
    use super::{ApiState, CapturedFrame, ClientMessage, ServerMessage};
    use std::net::SocketAddr;
    use std::sync::{Arc, Mutex};

    /// Shared list of per-client frame channels (module-level static).
    #[cfg(feature = "png")]
    static CLIENTS: std::sync::Mutex<Vec<tokio::sync::mpsc::UnboundedSender<CapturedFrame>>> 
        = std::sync::Mutex::new(Vec::new());

    /// Run the WebSocket server on the given address.
    pub async fn run(
        addr: SocketAddr,
        state: Arc<Mutex<ApiState>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use tokio::net::TcpListener;

        let listener = TcpListener::bind(addr).await?;
        eprintln!("[API] WebSocket server listening on ws://{}", addr);

        // Spawn frame distributor task: reads from global channel, distributes to clients
        #[cfg(feature = "png")]
        {
            use super::FRAME_RX;
            if let Some(rx) = *FRAME_RX.lock().unwrap() {
                tokio::spawn(frame_distributor_task(rx));
            }
        }

        loop {
            let (stream, peer) = listener.accept().await?;
            eprintln!("[API] Client connected: {}", peer);

            // Clone state for this client handler
            let state_clone = Arc::clone(&state);

            // Use async tungstenite directly on the Tokio stream
            tokio::spawn(async move {
                if let Err(e) = handle_client(stream, state_clone).await {
                    eprintln!("[API] Client error: {}", e);
                }
            });
        }
    }

    /// Frame distributor task: reads from global channel and broadcasts to clients.
    #[cfg(feature = "png")]
    async fn frame_distributor_task(rx: crossbeam_channel::Receiver<CapturedFrame>) {
        loop {
            // Read next frame (blocking)
            if let Ok(frame) = rx.recv() {
                // Broadcast to all connected clients
                let mut clients = CLIENTS.lock().unwrap();
                for client_tx in clients.iter_mut() {
                    let _ = client_tx.send_unbounded(frame.clone());
                }
                // Prune disconnected clients (channels that are closed)
                clients.retain(|tx| !tx.is_closed());
            }
        }
    }

    /// Handle a single WebSocket client connection.
    async fn handle_client(
        stream: tokio::net::TcpStream,
        state: Arc<Mutex<ApiState>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use futures_util::StreamExt;
        use tungstenite::Message;

        // Use async-tungstenite tokio module to accept the WebSocket handshake on a Tokio stream
        let mut ws = async_tungstenite::tokio::accept_async(stream).await?;

        #[cfg(feature = "png")]
        {
            use super::FRAME_RX;
            
            // Subscribe this client to frame distribution
            if let Some(ref _rx) = *FRAME_RX.lock().unwrap() {
                // Per-client channel for frame delivery
                let (client_tx, mut client_rx) = tokio::sync::mpsc::unbounded_channel::<CapturedFrame>();
                
                // Register client in global list
                {
                    let mut clients = CLIENTS.lock().unwrap();
                    clients.push(client_tx);
                }

                // Frame sender task (runs alongside message handling)
                let frame_sender = tokio::spawn(async move {
                    use std::time::Duration;
                    
                    // Rate limit: ~30fps max for PNG streaming
                    let mut last_send = std::time::Instant::now();
                    let min_frame_interval = Duration::from_millis(33); // ~30fps

                    loop {
                        match client_rx.recv().await {
                            Some(frame) => {
                                // Rate limit: skip if too soon since last send
                                if last_send.elapsed() < min_frame_interval {
                                    continue; // Drop frame to maintain target FPS
                                }
                                
                                if let Err(e) = send_png_frame(&mut ws, &frame).await {
                                    eprintln!("[API] Frame send error: {}", e);
                                    break;
                                }
                                last_send = std::time::Instant::now();
                            }
                            None => break, // Channel closed
                        }
                    }
                });

                // Main message handling loop (runs in parallel with frame sending)
                let msg_loop = async {
                    loop {
                        match ws.next().await {
                            Some(Ok(msg)) => {
                                if msg.is_text() {
                                    if let Err(e) = handle_message(&msg.to_string(), &state, &mut ws).await {
                                        eprintln!("[API] Message error: {}", e);
                                    }
                                } else if msg.is_close() || msg.is_ping() {
                                    break;
                                }
                            }
                            Some(Err(e)) => {
                                eprintln!("[API] Read error: {}", e);
                                break;
                            }
                            None => {
                                eprintln!("[API] Client disconnected");
                                break;
                            }
                        }
                    }
                };

                // Run both tasks - cancel frame sender when client disconnects
                tokio::select! {
                    _ = msg_loop => {},
                    _ = frame_sender => {},
                }
            } else {
                // No frame channel available, just handle messages
                loop {
                    match ws.next().await {
                        Some(Ok(msg)) => {
                            if msg.is_text() {
                                if let Err(e) = handle_message(&msg.to_string(), &state, &mut ws).await {
                                    eprintln!("[API] Message error: {}", e);
                                }
                            } else if msg.is_close() || msg.is_ping() {
                                break;
                            }
                        }
                        Some(Err(e)) => {
                            eprintln!("[API] Read error: {}", e);
                            break;
                        }
                        None => {
                            eprintln!("[API] Client disconnected");
                            break;
                        }
                    }
                }
            }
        }

        #[cfg(not(feature = "png"))]
        {
            loop {
                match ws.next().await {
                    Some(Ok(msg)) => {
                        if msg.is_text() {
                            if let Err(e) = handle_message(&msg.to_string(), &state, &mut ws).await {
                                eprintln!("[API] Message error: {}", e);
                            }
                        } else if msg.is_close() || msg.is_ping() {
                            break;
                        }
                    }
                    Some(Err(e)) => {
                        eprintln!("[API] Read error: {}", e);
                        break;
                    }
                    None => {
                        eprintln!("[API] Client disconnected");
                        break;
                    }
                }
            }
        }

        Ok(())
    }

    /// Encode a captured frame as PNG and send it over WebSocket.
    #[cfg(feature = "png")]
    async fn send_png_frame(
        ws: &mut async_tungstenite::WebSocketStream<async_tungstenite::tokio::TokioAdapter<tokio::net::TcpStream>>,
        frame: &CapturedFrame,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use tungstenite::Message;

        // Encode RGBA → PNG using the png crate
        let mut encoder = png::Encoder::new(Vec::new(), frame.width as u32, frame.height as u32);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap();
        writer.write_image_data(&frame.pixels).unwrap();
        let png_data = writer.finish().unwrap();

        // Build binary message: [width u16 BE][height u16 BE][PNG bytes]
        let header = [(frame.width >> 8) as u8, (frame.width & 0xFF) as u8,
                       (frame.height >> 8) as u8, (frame.height & 0xFF) as u8];
        let mut binary = Vec::with_capacity(4 + png_data.len());
        binary.extend_from_slice(&header);
        binary.extend_from_slice(&png_data);

        ws.send(Message::Binary(binary)).await?;
        Ok(())
    }

    async fn handle_message(
        text: &str,
        state: &Arc<Mutex<ApiState>>,
        ws: &mut async_tungstenite::WebSocketStream<async_tungstenite::tokio::TokioAdapter<tokio::net::TcpStream>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use tungstenite::Message;

        let msg = serde_json::from_str::<ClientMessage>(text)?;

        match msg {
            ClientMessage::Input { port, buttons } => {
                state.lock().unwrap().inputs.get_mut(port).clone_from(&buttons);
            }
            ClientMessage::Step => {
                state.lock().unwrap().request_frame_step();
                let resp = ServerMessage::FrameDone;
                ws.send(Message::text(serde_json::to_string(&resp)?)).await?;
            }
            ClientMessage::Play => {
                { let mut inner = state.lock().unwrap(); inner.start_playback(); }
                let (running, fps, width, height) = state.lock().unwrap().get_status();
                let resp = ServerMessage::Status { running, fps, width, height };
                ws.send(Message::text(serde_json::to_string(&resp)?)).await?;
            }
            ClientMessage::Pause => {
                { let mut inner = state.lock().unwrap(); inner.pause_emulation(); }
                let (running, fps, width, height) = state.lock().unwrap().get_status();
                let resp = ServerMessage::Status { running, fps, width, height };
                ws.send(Message::text(serde_json::to_string(&resp)?)).await?;
            }
            ClientMessage::SaveState => {
                state.lock().unwrap().request_save_state();
                let resp = ServerMessage::Flash { message: "Save Requested".into(), duration_ms: 2000 };
                ws.send(Message::text(serde_json::to_string(&resp)?)).await?;
            }
            ClientMessage::LoadState => {
                state.lock().unwrap().request_load_state();
                let resp = ServerMessage::Flash { message: "Load Requested".into(), duration_ms: 2000 };
                ws.send(Message::text(serde_json::to_string(&resp)?)).await?;
            }
            ClientMessage::SetOption { key, value } => {
                state.lock().unwrap().queue_option_change(key, value);
                let resp = ServerMessage::Flash { message: "Option Updated".into(), duration_ms: 1500 };
                ws.send(Message::text(serde_json::to_string(&resp)?)).await?;
            }
        }

        Ok(())
    }
}
