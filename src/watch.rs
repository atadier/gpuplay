use std::{ffi::OsString, fs::metadata, sync::mpsc::Sender, thread::sleep, time::Duration};

const RELOAD_INTERVAL: Duration = Duration::from_millis(500);

pub struct FileReloadNotification;

fn as_string(path: OsString) -> String {
    path.into_string()
        .unwrap_or_else(|s| String::from_utf8_lossy(s.as_encoded_bytes()).to_string())
}

pub fn send_reload(path: OsString, tx: Sender<FileReloadNotification>) {
    let mut last_mtime = None;
    loop {
        sleep(RELOAD_INTERVAL);

        match metadata(&path) {
            Ok(metadata) => {
                let mtime_opt = metadata.modified();
                if let Err(e) = mtime_opt {
                    eprintln!("cannot stat file '{}': {}", as_string(path), e);
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
                eprintln!("cannot open file '{}': {}", as_string(path), e);
                return;
            }
        }
    }
}
