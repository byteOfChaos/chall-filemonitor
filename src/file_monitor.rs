use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use crate::file_list::{FileEntry, FileList};

#[derive(Debug)]
pub enum FileEvent {
    New,
    Mod,
    Del,
}

pub struct EventEntry {
    event: FileEvent,
    entry: FileEntry,
}

impl std::fmt::Display for FileEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            Self::New => "NEW",
            Self::Del => "DEL",
            Self::Mod => "MOD",
        };

        write!(f, "{msg}")
    }
}

/// List contents of Inbox, takes flag to print directories, else prints only files
/// TODO: Determine/handle local system timezone instead of hard-coded UTC
/// Design is functional - is there any benefit to making it a member function?
pub async fn ls_inbox(entries: &Vec<FileEntry>, ls_dirs: bool) -> std::io::Result<()> {
    println!();
    for entry in entries {
        if ls_dirs || !entry.is_dir {
            let date_time: chrono::DateTime<chrono::Utc> = entry.last_mod.into();
            println!("[{}] {}", date_time.format("%F %T %Z"), entry);
        }
    }

    Ok(())
}

/// Periodically check for changes to Inbox. Using tokio per SOW.
/// TODO: Possibly future improvement: use notify crate
/// Design is functional - is there any benefit to making it a member function?
pub async fn monitor_inbox(inbox: &PathBuf, interval: Duration) -> std::io::Result<Vec<FileEntry>> {
    let running = Arc::new(AtomicBool::new(true));
    let run_flag = running.clone();

    ctrlc::set_handler(move || {
        run_flag.store(false, Ordering::SeqCst);
    })
    .expect("Failed to set Ctrl-C handler");

    let mut fl = FileList::new(inbox);
    let mut entries = fl.get_inbox_entries(true).await?;

    while running.load(Ordering::SeqCst) {
        tokio::time::sleep(interval).await;

        let mut events: Vec<EventEntry> = Vec::new();

        fl.reset();
        let new_entries = fl.get_inbox_entries(true).await?;

        // TODO: Currently filtering out diffs on directories - could be added as an option
        // FIXME: Massive overkill on iterations - optimize later
        for entry_prev in entries.iter().filter(|&e| !e.is_dir) {
            match new_entries
                .iter()
                .position(|entry_new| entry_new.path == entry_prev.path)
            {
                Some(index) => {
                    #[cfg(debug_assertions)]
                    println!(
                        "found {:?} ({:?}) at index {}",
                        entry_prev.path, new_entries[index].path, index
                    );

                    if new_entries[index].last_mod != entry_prev.last_mod {
                        #[cfg(debug_assertions)]
                        println!(
                            "mod {path:?}: {old:?} -> {new:?}",
                            path = entry_prev.path,
                            old = entry_prev.last_mod,
                            new = new_entries[index].last_mod
                        );

                        events.push(EventEntry {
                            event: FileEvent::Mod,
                            entry: entry_prev.clone(),
                        });
                    }
                }
                None => {
                    #[cfg(debug_assertions)]
                    println!("not found: {:?}", entry_prev.path);

                    events.push(EventEntry {
                        event: FileEvent::Del,
                        entry: entry_prev.clone(),
                    });
                }
            }
        }

        for new_entry in &new_entries {
            if !entries.iter().any(|e| e.path == new_entry.path) {
                events.push(EventEntry {
                    event: FileEvent::New,
                    entry: new_entry.clone(),
                })
            }
        }

        events.sort_by_key(|e| e.entry.path.clone());

        entries = new_entries;

        for event in events {
            println!("[{}] {:?}", event.event, event.entry.path);
        }
    }

    Ok(entries)
}
