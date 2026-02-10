use std::collections::HashMap;
use shared::{AppEvent, NotificationLevel, SystemNotification, Torrent, TorrentUpdate};

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

        // Manuel diff creating TorrentUpdate (which is the Patch struct)
        let mut patch = TorrentUpdate::default();
        let mut has_changes = false;

        if old_t.name != new_t.name { patch.name = Some(new_t.name.clone()); has_changes = true; }
        if old_t.size != new_t.size { patch.size = Some(new_t.size); has_changes = true; }
        if old_t.down_rate != new_t.down_rate { patch.down_rate = Some(new_t.down_rate); has_changes = true; }
        if old_t.up_rate != new_t.up_rate { patch.up_rate = Some(new_t.up_rate); has_changes = true; }
        if old_t.completed != new_t.completed { patch.completed = Some(new_t.completed); has_changes = true; }
        if old_t.eta != new_t.eta { patch.eta = Some(new_t.eta); has_changes = true; }
        if (old_t.percent_complete - new_t.percent_complete).abs() > 0.01 { 
            patch.percent_complete = Some(new_t.percent_complete); 
            has_changes = true; 
            
            if old_t.percent_complete < 100.0 && new_t.percent_complete >= 100.0 {
                events.push(AppEvent::Notification(SystemNotification {
                    level: NotificationLevel::Success,
                    message: format!("Torrent tamamlandı: {}", new_t.name),
                }));
            }
        }
        if old_t.status != new_t.status { patch.status = Some(new_t.status.clone()); has_changes = true; }
        if old_t.error_message != new_t.error_message { patch.error_message = Some(new_t.error_message.clone()); has_changes = true; }
        if old_t.label != new_t.label { patch.label = Some(new_t.label.clone()); has_changes = true; }

        if has_changes {
            // Set the hash (not an Option in Patch usually, but check shared/src/lib.rs)
            // Wait, TorrentUpdate is a Patch, does it have 'hash' field?
            // Yes, because Torrent has 'hash' field.
            patch.hash = Some(new_t.hash.clone()); 
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