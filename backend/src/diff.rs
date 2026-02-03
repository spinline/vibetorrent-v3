use shared::{AppEvent, Torrent, TorrentUpdate};

#[derive(Debug)]
pub enum DiffResult {
    NoChange,
    FullUpdate,
    Partial(Vec<AppEvent>),
}

pub fn diff_torrents(old: &[Torrent], new: &[Torrent]) -> DiffResult {
    // 1. Structural Check (Length or Order changed)
    if old.len() != new.len() {
        return DiffResult::FullUpdate;
    }

    for (i, t) in new.iter().enumerate() {
        if old[i].hash != t.hash {
            return DiffResult::FullUpdate;
        }
    }

    // 2. Field Updates
    let mut events = Vec::new();

    for (i, new_t) in new.iter().enumerate() {
        let old_t = &old[i];

        // Initialize with all None
        let mut update = TorrentUpdate {
            hash: new_t.hash.clone(),
            name: None,
            size: None,
            down_rate: None,
            up_rate: None,
            percent_complete: None,
            completed: None,
            eta: None,
            status: None,
            error_message: None,
            label: None,
        };

        let mut has_changes = false;

        // Compare fields
        if old_t.name != new_t.name {
            update.name = Some(new_t.name.clone());
            has_changes = true;
        }
        if old_t.size != new_t.size {
            update.size = Some(new_t.size);
            has_changes = true;
        }
        if old_t.down_rate != new_t.down_rate {
            update.down_rate = Some(new_t.down_rate);
            has_changes = true;
        }
        if old_t.up_rate != new_t.up_rate {
            update.up_rate = Some(new_t.up_rate);
            has_changes = true;
        }
        if (old_t.percent_complete - new_t.percent_complete).abs() > 0.01 {
            update.percent_complete = Some(new_t.percent_complete);
            has_changes = true;
        }
        if old_t.completed != new_t.completed {
            update.completed = Some(new_t.completed);
            has_changes = true;
        }
        if old_t.eta != new_t.eta {
            update.eta = Some(new_t.eta);
            has_changes = true;
        }
        if old_t.status != new_t.status {
            update.status = Some(new_t.status.clone());
            has_changes = true;
        }
        if old_t.error_message != new_t.error_message {
            update.error_message = Some(new_t.error_message.clone());
            has_changes = true;
        }
        if old_t.label != new_t.label {
            update.label = new_t.label.clone();
            has_changes = true;
        }

        if has_changes {
            events.push(AppEvent::Update(update));
        }
    }

    if events.is_empty() {
        DiffResult::NoChange
    } else {
        tracing::debug!("Generated {} partial updates", events.len());
        DiffResult::Partial(events)
    }
}
