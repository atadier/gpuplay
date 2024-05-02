use std::{
    ffi::OsString,
    fs::metadata,
    sync::mpsc::{Receiver, Sender},
    thread::sleep,
    time::{Duration, SystemTime},
};

const RELOAD_INTERVAL: Duration = Duration::from_millis(500);

pub struct FileReloadNotification;

pub struct FileReloadSetPath(pub OsString);

pub fn send_reload(tx: Sender<FileReloadNotification>, rx: Receiver<FileReloadSetPath>) {
    let mut watching_path: Option<OsString> = None;
    let mut last_mtime: Option<SystemTime> = None;

    loop {
        sleep(RELOAD_INTERVAL);

        if let Some(ref path) = watching_path {
            match metadata(&path) {
                Ok(metadata) => {
                    let mtime_opt = metadata.modified();
                    if let Err(e) = mtime_opt {
                        eprintln!("cannot stat file '{}': {}", path.to_string_lossy(), e);
                        return;
                    }
                    let mtime = mtime_opt.unwrap();

                    if last_mtime.is_some_and(|last| mtime > last) {
                        tx.send(FileReloadNotification)
                            .expect("failed to send notification");
                    }
                    last_mtime = Some(mtime);
                }
                Err(e) => {
                    eprintln!("cannot open file '{}': {}", path.to_string_lossy(), e);
                    watching_path = None;
                }
            }
        }

        match rx.try_recv() {
            Ok(FileReloadSetPath(new_path)) => {
                watching_path = Some(new_path);
                last_mtime = None;
            }
            _ => (),
        }
    }
}
