#[macro_use]
extern crate log;

pub mod play;
pub mod server;

use android_logger::Config;
use log::LevelFilter;
use music_player_settings::{read_settings, Settings};
use std::thread;
use walkdir::WalkDir;

use crate::play::play;

#[no_mangle]
#[export_name = "Java_com_tsirysndr_songbird_Songbird_00024Companion_start"]
pub extern "C" fn start() {
    android_logger::init_once(Config::default().with_max_level(LevelFilter::Trace));
    debug!(
        ">> config dir {}",
        dirs::config_dir().unwrap().to_str().unwrap()
    );
    debug!(
        r#"
    __  ___           _      ____  __                     
   /  |/  /_  _______(_)____/ __ \/ /___ ___  _____  _____
  / /|_/ / / / / ___/ / ___/ /_/ / / __ `/ / / / _ \/ ___/
 / /  / / /_/ (__  ) / /__/ ____/ / /_/ / /_/ /  __/ /    
/_/  /_/\__,_/____/_/\___/_/   /_/\__,_/\__, /\___/_/     
                                       /____/             

A simple music player written in Rust"#
    );
    debug!("Starting server in a new thread");
    thread::spawn(|| {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        runtime.block_on(server::start_all()).unwrap();
    });
}

#[no_mangle]
#[export_name = "Java_com_tsirysndr_songbird_Songbird_00024Companion_start_1blocking"]
pub extern "C" fn start_blocking() {
    android_logger::init_once(Config::default().with_max_level(LevelFilter::Trace));
    debug!(
        r#"
    __  ___           _      ____  __                     
   /  |/  /_  _______(_)____/ __ \/ /___ ___  _____  _____
  / /|_/ / / / / ___/ / ___/ /_/ / / __ `/ / / / _ \/ ___/
 / /  / / /_/ (__  ) / /__/ ____/ / /_/ / /_/ /  __/ /    
/_/  /_/\__,_/____/_/\___/_/   /_/\__,_/\__, /\___/_/     
                                       /____/             

A simple music player written in Rust"#
    );
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    runtime.block_on(server::start_all()).unwrap();
}

#[no_mangle]
#[export_name = "Java_com_tsirysndr_songbird_Songbird_00024Companion_example"]
pub extern "C" fn example() {
    android_logger::init_once(Config::default().with_max_level(LevelFilter::Trace));
    debug!("Hello Android!");
    debug!("this is a debug {}", "message");
    error!("this is printed by default");
    let config = match read_settings() {
        Ok(config) => config,
        Err(e) => {
            error!("Error reading config: {}", e);
            return;
        }
    };
    let settings = config.try_deserialize::<Settings>().unwrap();
    debug!(">> scanning music library...");
    for (_, entry) in WalkDir::new(&settings.music_directory)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
        .enumerate()
    {
        let path = format!("{}", entry.path().display());
        debug!(">> {}", path);
    }
    debug!(">> done scanning music library");
    /* 
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    runtime.block_on(play());
    */
}
