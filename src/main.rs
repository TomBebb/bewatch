// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::cell::LazyCell;
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::PathBuf;

slint::include_modules!();

use bincode::enc::write::SliceWriter;
use robius_directories::ProjectDirs;

const DIRS: LazyCell<ProjectDirs> =
    LazyCell::new(|| ProjectDirs::from("com", "TomBebb", "bewatch").unwrap());
const SETTINGS_DIR: LazyCell<PathBuf> = LazyCell::new(|| DIRS.config_local_dir().into());
const SETTINGS_PATH: LazyCell<PathBuf> = LazyCell::new(|| SETTINGS_DIR.join("settings.bin"));

fn load_settings(ui: &AppWindow) {
    println!("Loading settings: {}", SETTINGS_PATH.display());
    let file = File::open(&*SETTINGS_PATH).unwrap();
    let reader = BufReader::new(file);
    let loaded_settings: MySettings =
        bincode::decode_from_reader(reader, bincode::config::standard())
            .expect("invalid settings file");
    println!("Loaded settings: {:?}", loaded_settings);
    let settings = ui.global::<Settings>();
    settings.set_value(loaded_settings);
    settings.invoke_change(settings.get_value());
}

fn save_settings(settings: &MySettings) {
    let mut file = File::create(&*SETTINGS_PATH).unwrap();
    let mut buf = Box::new([0u8; 512]);
    bincode::encode_into_writer(
        settings,
        SliceWriter::new(&mut *buf),
        bincode::config::standard(),
    )
    .unwrap();

    println!("Buf size: {}", buf.len());
    file.write_all(&*buf).unwrap();
}
fn main() -> Result<(), Box<dyn Error>> {
    std::fs::create_dir_all(&*SETTINGS_DIR)?;
    let ui = AppWindow::new()?;
    ui.on_request_increase_value({
        let ui_handle = ui.as_weak();
        move || {
            let ui = ui_handle.unwrap();
            ui.set_counter(ui.get_counter() + 1);
        }
    });

    let settings = ui.global::<Settings>();

    ui.global::<CurrentVideo>()
        .on_load(|url| println!("Load URL: {}", url));

    ui.global::<CurrentVideo>().set_url("https://test-videos.co.uk/vids/bigbuckbunny/mp4/h264/1080/Big_Buck_Bunny_1080_10s_10MB.mp4".into());
    println!("Settings: {:?}", settings.get_value());
    settings.on_change(move |settings| {
        println!("OnChange: {:?}", settings);
        save_settings(&settings);
    });

    if SETTINGS_PATH.exists() {
        load_settings(&ui);
    }

    ui.run()?;
    Ok(())
}
