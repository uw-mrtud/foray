use std::ffi::OsStr;
use std::path::Component;
use std::path::PathBuf;
use std::time::Duration;

use iced::futures::sink::SinkExt;
use iced::stream;
use iced::Subscription;
use log::trace;
use notify::RecursiveMode;
use notify_debouncer_full::new_debouncer;

use crate::app::Message;

/// Sends Message::ReloadNodes on a python file change
pub fn file_watch_subscription(id: usize, nodes_dir: PathBuf) -> Subscription<Message> {
    let stream = stream::channel(0, |mut output| async move {
        trace!("Starting file watch subscription stream: {nodes_dir:?}");
        let (sender, receiver) = std::sync::mpsc::channel();
        let mut debouncer = new_debouncer(Duration::from_millis(250), None, sender).unwrap();
        debouncer
            .watch(nodes_dir, RecursiveMode::Recursive)
            .unwrap();

        for res in receiver {
            match res {
                Ok(events) => {
                    let nodes: Vec<_> = events
                        .into_iter()
                        .map(|debounce_event| debounce_event.event)
                        .filter(|e| {
                            (e.kind.is_modify() || e.kind.is_create())
                                && e.paths.iter().any(|p| {
                                    p.extension() == Some(OsStr::new("py"))
                                        && !p
                                            .components()
                                            .any(|s| s == Component::Normal(OsStr::new(".venv")))
                                    //TODO: more reliable check here?
                                })
                        })
                        .collect();
                    if !nodes.is_empty() {
                        let _ = output.send(Message::ReloadNodes).await;
                    }
                }
                Err(error) => log::error!("Error: {error:?}"),
            }
        }
    });
    Subscription::run_with_id(id, stream)
}
