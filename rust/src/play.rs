use futures_channel::mpsc::UnboundedSender;
use music_player_playback::{
    audio_backend::{self, rodio::RodioSink},
    config::AudioFormat,
    player::{Player, PlayerEngine},
};
// use music_player_server::event::{Event, TrackEvent};
use music_player_tracklist::Tracklist;
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{self, Arc, Mutex},
};
use tungstenite::Message;

type Tx = UnboundedSender<Message>;
type PeerMap = Arc<sync::Mutex<HashMap<SocketAddr, Tx>>>;

pub async fn play() {
    let audio_format = AudioFormat::default();
    let backend = audio_backend::find(Some(RodioSink::NAME.to_string())).unwrap();
    let (cmd_tx, cmd_rx) = tokio::sync::mpsc::unbounded_channel();
    let cmd_tx = Arc::new(Mutex::new(cmd_tx));
    let cmd_rx = Arc::new(Mutex::new(cmd_rx));
    let tracklist = Arc::new(Mutex::new(Tracklist::new_empty()));
    let (mut player, _) = Player::new(
        move || backend(None, audio_format),
        |_| {},
        cmd_tx,
        cmd_rx,
        tracklist,
    );

    // let song = "/storage/emulated/0/Music/mp3/02 Don't Stay.m4a";
    let song = "https://raw.githubusercontent.com/tsirysndr/music-player/master/fixtures/audio/06%20-%20J.%20Cole%20-%20Fire%20Squad(Explicit).m4a";
    /*
    let url = "https://raw.githubusercontent.com/tsirysndr/music-player/master/fixtures/audio/06%20-%20J.%20Cole%20-%20Fire%20Squad(Explicit).m4a";
    // let url = "/tmp/audio/06 - J. Cole - Fire Squad(Explicit).m4a";
    let bytes_per_second = 40 * 1024; // 320kbps
    let audio_file = match AudioFile::open(url, bytes_per_second).await {
        Ok(audio_file) => audio_file,
        Err(e) => {
            println!("Error: {}", e);
            return Ok(());
        }
    };
    */
    debug!("Playing {}", song);
    player.load(song, true, 0);
    player.await_end_of_track().await;
    debug!("End of track");
}
