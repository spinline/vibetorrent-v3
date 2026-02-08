use std::collections::HashMap;
use shared::{AppEvent, NotificationLevel, SystemNotification, Torrent, TorrentUpdate};

#[derive(Debug)]
pub enum DiffResult {
    NoChange,
    FullUpdate,
    Partial(Vec<AppEvent>),
}

pub fn diff_torrents(old: &[Torrent], new: &[Torrent]) -> DiffResult {
    // 1. Structural Check: Eğer torrent sayısı değişmişse (yeni eklenen veya silinen), 
    // şimdilik basitlik adına FullUpdate gönderiyoruz.
    if old.len() != new.len() {
        return DiffResult::FullUpdate;
    }

    // 2. Hash Set Karşılaştırması: 
    // Sıralama değişmiş olabilir ama torrentler aynı mı?
    let old_map: HashMap<&str, &Torrent> = old.iter().map(|t| (t.hash.as_str(), t)).collect();
    
    // Eğer yeni listedeki bir hash eski listede yoksa, yapı değişmiş demektir.
    for new_t in new {
        if !old_map.contains_key(new_t.hash.as_str()) {
            return DiffResult::FullUpdate;
        }
    }

    // 3. Alan Güncellemeleri (Partial Updates)
    // Buraya geldiğimizde biliyoruz ki old ve new listelerindeki torrentler (hash olarak) aynı,
    // sadece sıraları farklı olabilir veya içindeki veriler güncellenmiş olabilir.
    let mut events = Vec::new();

    for new_t in new {
        // old_map'ten ilgili torrente hash ile ulaşalım (sıradan bağımsız)
        let old_t = old_map.get(new_t.hash.as_str()).unwrap();

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

        // Alanları karşılaştır
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

            // Torrent tamamlanma kontrolü
            if old_t.percent_complete < 100.0 && new_t.percent_complete >= 100.0 {
                tracing::info!("Torrent completed: {} ({})", new_t.name, new_t.hash);
                events.push(AppEvent::Notification(SystemNotification {
                    level: NotificationLevel::Success,
                    message: format!("Torrent tamamlandı: {}", new_t.name),
                }));
            }
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
            
            tracing::debug!(
                "Torrent status changed: {} ({}) {:?} -> {:?}",
                new_t.name, new_t.hash, old_t.status, new_t.status
            );
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