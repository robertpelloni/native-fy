use notify::{Watcher, RecursiveMode, Result};
use std::sync::mpsc::{channel, Sender};
use crate::runtime::UiCommand;
use std::path::Path;

pub fn start_hot_reloader(ui_tx: Sender<UiCommand>) -> Result<notify::RecommendedWatcher> {
    let (tx, rx) = channel();

    let mut watcher = notify::recommended_watcher(tx)?;

    // We watch the scripts directory and the ui_ast.json file
    let _ = watcher.watch(Path::new("src/runtime.js"), RecursiveMode::NonRecursive);
    let _ = watcher.watch(Path::new("ui_ast.json"), RecursiveMode::NonRecursive);

    std::thread::spawn(move || {
        for res in rx {
            match res {
                Ok(event) => {
                    // Filter duplicate events by checking if it's a modify
                    if event.kind.is_modify() {
                        println!("[Hot-Reload] File change detected: {:?}. Triggering engine reload.", event.paths);
                        let _ = ui_tx.send(UiCommand::Reload);
                    }
                },
                Err(e) => println!("watch error: {:?}", e),
            }
        }
    });

    Ok(watcher)
}
