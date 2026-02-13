use leptos::prelude::*;
use crate::components::ui::context_menu::{
    ContextMenu, ContextMenuContent, ContextMenuItem, ContextMenuTrigger,
};
use crate::components::ui::button_action::ButtonAction;
use crate::components::ui::button::ButtonVariant;

#[component]
pub fn TorrentContextMenu(
    children: Children,
    torrent_hash: String,
    on_action: Callback<(String, String)>,
) -> impl IntoView {
    let hash_c1 = torrent_hash.clone();
    let hash_c2 = torrent_hash.clone();
    let hash_c3 = torrent_hash.clone();
    let hash_c4 = torrent_hash.clone();
    
    let on_action_stored = StoredValue::new(on_action);

    view! {
        <ContextMenu>
            <ContextMenuTrigger>
                {children()}
            </ContextMenuTrigger>
            <ContextMenuContent class="w-56 p-1.5">
                <ContextMenuItem on:click={let h = hash_c1; move |_| {
                    on_action_stored.get_value().run(("start".to_string(), h.clone()));
                    crate::components::ui::context_menu::close_context_menu();
                }}>
                    "Başlat"
                </ContextMenuItem>
                <ContextMenuItem on:click={let h = hash_c2; move |_| {
                    on_action_stored.get_value().run(("stop".to_string(), h.clone()));
                    crate::components::ui::context_menu::close_context_menu();
                }}>
                    "Durdur"
                </ContextMenuItem>
                
                <div class="my-1.5 h-px bg-border/50" />
                
                // --- Modern Hold-to-Action Buttons ---
                <div class="space-y-1">
                    <ButtonAction 
                        variant=ButtonVariant::Ghost
                        class="w-full justify-start h-8 px-2 text-xs text-destructive hover:bg-destructive/10 hover:text-destructive transition-none"
                        hold_duration=800
                        on_action={let h = hash_c3; move || {
                            on_action_stored.get_value().run(("delete".to_string(), h.clone()));
                            crate::components::ui::context_menu::close_context_menu();
                        }}
                    >
                        "Sil (Basılı Tut)"
                    </ButtonAction>

                    <ButtonAction 
                        variant=ButtonVariant::Destructive
                        class="w-full justify-start h-8 px-2 text-xs font-bold"
                        hold_duration=1200
                        on_action={let h = hash_c4; move || {
                            on_action_stored.get_value().run(("delete_with_data".to_string(), h.clone()));
                            crate::components::ui::context_menu::close_context_menu();
                        }}
                    >
                        "Verilerle Sil (Basılı Tut)"
                    </ButtonAction>
                </div>
            </ContextMenuContent>
        </ContextMenu>
    }
}
