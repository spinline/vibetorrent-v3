use std::collections::HashMap;
use shared::{AppEvent, NotificationLevel, SystemNotification, Torrent};
use struct_patch::traits::Patchable;

#[derive(Debug)]
pub enum DiffResult {
    NoChange,
    FullUpdate,
    Partial(Vec<AppEvent>),
}

pub fn diff_torrents(old: &[Torrent], new: &[Torrent]) -> DiffResult {
    if old.len() != new.len() {
        return DiffResult::FullUpdate;
    }

    let old_map: HashMap<&str, &Torrent> = old.iter().map(|t| (t.hash.as_str(), t)).collect();
    
    for new_t in new {
        if !old_map.contains_key(new_t.hash.as_str()) {
            return DiffResult::FullUpdate;
        }
    }

    let mut events = Vec::new();

    for new_t in new {
        let old_t = old_map.get(new_t.hash.as_str()).unwrap();

        // struct_patch::diff uses the Patch trait we derived in shared crate
        let patch = old_t.diff(new_t);

        if !patch.is_empty() {
            // If percent_complete jumped to 100, send notification
            if old_t.percent_complete < 100.0 && new_t.percent_complete >= 100.0 {
                tracing::info!("Torrent completed: {} ({})", new_t.name, new_t.hash);
                events.push(AppEvent::Notification(SystemNotification {
                    level: NotificationLevel::Success,
                    message: format!("Torrent tamamlandı: {}", new_t.name),
                }));
            }
            events.push(AppEvent::Update(patch));
        }
    }

    if events.is_empty() {
        DiffResult::NoChange
    } else {
        tracing::debug!("Generated {} partial updates", events.len());
        DiffResult::Partial(events)
    }
}