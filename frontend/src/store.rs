use futures::StreamExt;
use gloo_net::eventsource::futures::EventSource;
use leptos::*;
use shared::{AppEvent, Torrent};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FilterStatus {
    All,
    Downloading,
    Seeding,
    Completed,
    Inactive,
    Active,
    Error,
}

impl FilterStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            FilterStatus::All => "All",
            FilterStatus::Downloading => "Downloading",
            FilterStatus::Seeding => "Seeding",
            FilterStatus::Completed => "Completed",
            FilterStatus::Inactive => "Inactive",
            FilterStatus::Active => "Active",
            FilterStatus::Error => "Error",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct TorrentStore {
    pub torrents: RwSignal<Vec<Torrent>>,
    pub filter: RwSignal<FilterStatus>,
    pub search_query: RwSignal<String>,
}

pub fn provide_torrent_store() {
    let torrents = create_rw_signal(Vec::<Torrent>::new());
    let filter = create_rw_signal(FilterStatus::All);
    let search_query = create_rw_signal(String::new());

    let store = TorrentStore {
        torrents,
        filter,
        search_query,
    };
    provide_context(store);

    // Initialize SSE connection
    create_effect(move |_| {
        spawn_local(async move {
            let mut es = EventSource::new("/api/events").unwrap();
            let mut stream = es.subscribe("message").unwrap();

            while let Some(Ok((_, msg))) = stream.next().await {
                if let Some(data_str) = msg.data().as_string() {
                    if let Ok(event) = serde_json::from_str::<AppEvent>(&data_str) {
                        match event {
                            AppEvent::FullList { torrents: list, .. } => {
                                torrents.set(list);
                            }
                            AppEvent::Update(update) => {
                                torrents.update(|list| {
                                    if let Some(t) = list.iter_mut().find(|t| t.hash == update.hash)
                                    {
                                        if let Some(name) = update.name {
                                            t.name = name;
                                        }
                                        if let Some(size) = update.size {
                                            t.size = size;
                                        }
                                        if let Some(down_rate) = update.down_rate {
                                            t.down_rate = down_rate;
                                        }
                                        if let Some(up_rate) = update.up_rate {
                                            t.up_rate = up_rate;
                                        }
                                        if let Some(percent_complete) = update.percent_complete {
                                            t.percent_complete = percent_complete;
                                        }
                                        if let Some(completed) = update.completed {
                                            t.completed = completed;
                                        }
                                        if let Some(eta) = update.eta {
                                            t.eta = eta;
                                        }
                                        if let Some(status) = update.status {
                                            t.status = status;
                                        }
                                        if let Some(error_message) = update.error_message {
                                            t.error_message = error_message;
                                        }
                                    }
                                });
                            }
                        }
                    }
                }
            }
        });
    });
}
