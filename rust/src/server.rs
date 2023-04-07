use futures_channel::mpsc::UnboundedSender;
use music_player_discovery::register_services;
use music_player_graphql::{
    schema::{
        objects::{player_state::PlayerState, track::Track},
        playback::PositionMilliseconds,
    },
    simple_broker::SimpleBroker,
};
use music_player_playback::{
    audio_backend::{self, rodio::RodioSink},
    config::AudioFormat,
    player::{Player, PlayerEvent},
};
use music_player_server::event::{Event, TrackEvent};
use music_player_server::server::MusicPlayerServer;
use music_player_storage::{searcher::Searcher, Database};
use music_player_tracklist::Tracklist;
use music_player_webui::start_webui;
use sea_orm::{ConnectionTrait, DbBackend, Statement};
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{self, Arc},
    thread,
};
use tokio::sync::Mutex;
use tungstenite::Message;

type Tx = UnboundedSender<Message>;
type PeerMap = Arc<sync::Mutex<HashMap<SocketAddr, Tx>>>;

pub async fn start_all() -> anyhow::Result<()> {
    migration::run().await;
    let audio_format = AudioFormat::default();
    let backend = audio_backend::find(Some(RodioSink::NAME.to_string())).unwrap();
    let peer_map: PeerMap = Arc::new(sync::Mutex::new(HashMap::new()));
    let cloned_peer_map = Arc::clone(&peer_map);

    let db = Database::new().await;
    let conn = db.get_connection();
    conn.execute(Statement::from_string(
        DbBackend::Sqlite,
        "PRAGMA case_sensitive_like=OFF;".to_owned(),
    ))
    .await?;
    db.create_indexes().await;

    // let db_conn = Database::new().await;
    // let searcher = Searcher::new();

    let db = Arc::new(Mutex::new(Database::new().await));
    let tracklist = Arc::new(std::sync::Mutex::new(Tracklist::new_empty()));
    let (cmd_tx, cmd_rx) = tokio::sync::mpsc::unbounded_channel();
    let cloned_tracklist = Arc::clone(&tracklist);
    let cmd_tx = Arc::new(std::sync::Mutex::new(cmd_tx));
    let cmd_rx = Arc::new(std::sync::Mutex::new(cmd_rx));
    let cloned_cmd_tx = Arc::clone(&cmd_tx);
    let cloned_cmd_rx = Arc::clone(&cmd_rx);
    let cmd_tx_ws = Arc::clone(&cloned_cmd_tx);
    let cmd_tx_webui = Arc::clone(&cloned_cmd_tx);
    let (_, _) = Player::new(
        move || backend(None, audio_format),
        move |event| {
            let peers = cloned_peer_map.lock().unwrap();

            let broadcast_recipients = peers.iter().map(|(_, ws_sink)| ws_sink);

            match event {
                PlayerEvent::CurrentTrack {
                    track,
                    position,
                    position_ms,
                    is_playing,
                } => {
                    if let Some(track) = track.clone() {
                        SimpleBroker::publish(Track::from(track));
                        SimpleBroker::publish(PlayerState {
                            index: position as u32,
                            position_ms,
                            is_playing,
                        });
                    }

                    let track_event = TrackEvent {
                        track,
                        index: position as u32,
                        is_playing,
                        position_ms,
                    };
                    let msg = Event {
                        event_type: "current_track".to_string(),
                        data: serde_json::to_string(&track_event).unwrap(),
                    };
                    for recp in broadcast_recipients {
                        recp.unbounded_send(Message::text(serde_json::to_string(&msg).unwrap()))
                            .unwrap();
                    }
                }
                PlayerEvent::TrackTimePosition { position_ms } => {
                    SimpleBroker::publish(PositionMilliseconds { position_ms });
                }
                _ => {}
            }
        },
        cloned_cmd_tx,
        cloned_cmd_rx,
        Arc::clone(&tracklist),
    );

    let tracklist_ws = Arc::clone(&tracklist);
    let tracklist_webui = Arc::clone(&tracklist);
    let db_ws = Arc::clone(&db);
    let peer_map_ws = Arc::clone(&peer_map);

    register_services();

    // Start the web server
    thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        match runtime.block_on(
            MusicPlayerServer::new(
                cloned_tracklist,
                Arc::clone(&cmd_tx),
                Arc::clone(&peer_map),
                db,
            )
            .start(),
        ) {
            Ok(_) => {}
            Err(e) => {
                panic!("{}", e);
            }
        }
    });
    // Spawn a thread to handle the player events
    thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        match runtime.block_on(
            MusicPlayerServer::new(tracklist_ws, cmd_tx_ws, peer_map_ws, db_ws).start_ws(),
        ) {
            Ok(_) => {}
            Err(e) => {
                println!("{}", e);
            }
        }
    });
    start_webui(cmd_tx_webui, tracklist_webui).await?;

    Ok(())
}
