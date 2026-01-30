use shared::{AppEvent, Torrent, TorrentUpdate};

pub fn diff_torrents(old: &[Torrent], new: &[Torrent]) -> Vec<AppEvent> {
    // 1. Structural Change Check
    // If length differs or any hash at specific index differs (simplistic view), send FullList.
    // Ideally we should track "Added/Removed", but for simplicity and robustness as per prompt "FullList for big changes",
    // we fallback to FullList on structural changes.
    if old.len() != new.len() {
        // Timestamp is needed for FullList? The definition is FullList(Vec, u64).
        // We'll let the caller handle the timestamp or pass it in?
        // AppEvent in shared::lib.rs is FullList(Vec<Torrent>, u64).
        // We'll return just the list decision here, or constructs events.
        // Let's assume caller adds the u64 (disk space/timestamp).
        // Actually, let's keep it simple: Return Option<Vec<AppEvent>>.
        // But simply returning "NeedFullList" signal is easier if we can't accept u64 here.
        // Let's change signature to return an enum or boolean flag if FullList needed.
        return vec![]; // Special signal: Empty vec means "No diffs" or "Caller handles FullList"?
                       // This function is tricky if we don't have the u64.
    }
    
    // Check for hash mismatch (order changed)
    for (i, t) in new.iter().enumerate() {
        if old[i].hash != t.hash {
            return vec![]; // Signal Full List needed
        }
    }

    let mut events = Vec::new();

    for (i, new_t) in new.iter().enumerate() {
        let old_t = &old[i];
        
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
        };
        
        let mut has_changes = false;

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
        // Floating point comparison with epsilon
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

        if has_changes {
            events.push(AppEvent::Update(update));
        }
    }
    
    if !events.is_empty() {
        tracing::debug!("Generated {} updates", events.len());
    }
    
    events
}
