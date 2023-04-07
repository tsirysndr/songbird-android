#[macro_use]
extern crate log;

use android_logger::Config;
use log::LevelFilter;

#[no_mangle]
#[export_name = "Java_com_tsirysndr_songbird_Songbird_00024Companion_example"]
pub extern "C" fn example() {
    android_logger::init_once(Config::default().with_max_level(LevelFilter::Trace));
    debug!("Hello Android!");
    debug!("this is a debug {}", "message");
    error!("this is printed by default");
}

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
