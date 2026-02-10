use leptos::prelude::*;
use leptos::portal::Portal;
use leptos_shadcn_context_menu::{
    ContextMenu,
    ContextMenuContent,
    ContextMenuItem,
    ContextMenuTrigger,
    ContextMenuSeparator,
};

#[component]
pub fn TorrentContextMenu(
    children: Children,
    torrent_hash: String,
    on_action: Callback<(String, String)>,
) -> impl IntoView {
    let hash = StoredValue::new(torrent_hash);
    let on_action = StoredValue::new(on_action);

    view! {
        <ContextMenu>
            <ContextMenuTrigger class="w-full">
                {children()}
            </ContextMenuTrigger>
            
            <Portal>
                <ContextMenuContent class="w-56 z-[100] bg-popover border border-border shadow-md rounded-md p-1">
                    <ContextMenuItem on:click=move |_| {
                        on_action.get_value().run(("start".to_string(), hash.get_value()));
                    }>
                        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="mr-2 h-4 w-4 opacity-70">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M5.25 5.653c0-.856.917-1.398 1.667-.986l11.54 6.348a1.125 1.125 0 010 1.971l-11.54 6.347a1.125 1.125 0 01-1.667-.985V5.653z" />
                        </svg>
                        "Start"
                    </ContextMenuItem>
                    
                    <ContextMenuItem on:click=move |_| {
                        on_action.get_value().run(("stop".to_string(), hash.get_value()));
                    }>
                        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="mr-2 h-4 w-4 opacity-70">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M15.75 5.25v13.5m-7.5-13.5v13.5" />
                        </svg>
                        "Stop"
                    </ContextMenuItem>

                    <ContextMenuItem on:click=move |_| {
                        on_action.get_value().run(("recheck".to_string(), hash.get_value()));
                    }>
                        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="mr-2 h-4 w-4 opacity-70">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M16.023 9.348h4.992v-.001M2.985 19.644v-4.992m0 0h4.992m-4.993 0l3.181 3.183a8.25 8.25 0 0013.803-3.7M4.031 9.865a8.25 8.25 0 0113.803-3.7l3.181 3.182m0-4.991v4.99" />
                        </svg>
                        "Recheck"
                    </ContextMenuItem>
                    
                    <ContextMenuSeparator />
                    
                    <ContextMenuItem 
                        class="text-destructive focus:text-destructive-foreground focus:bg-destructive"
                        on:click=move |_| {
                            on_action.get_value().run(("delete".to_string(), hash.get_value()));
                        }
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="mr-2 h-4 w-4 opacity-70">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M14.74 9l-.346 9m-4.788 0L9.26 9m9.968-3.21c.342.052.682.107 1.022.166m-1.022-.165L18.16 19.673a2.25 2.25 0 01-2.244 2.077H8.084a2.25 2.25 0 01-2.244-2.077L4.772 5.79m14.456 0a48.108 48.108 0 00-3.478-.397m-12 .562c.34-.059.68-.114 1.022-.165m0 0a48.11 48.11 0 013.478-.397m7.5 0v-.916c0-1.18-.91-2.164-2.09-2.164h-2.34c-1.18 0-2.09.984-2.09 2.164v.916m7.5 0a48.667 48.667 0 00-7.5 0" />
                        </svg>
                        "Remove"
                    </ContextMenuItem>

                    <ContextMenuItem 
                        class="text-destructive focus:text-destructive-foreground focus:bg-destructive"
                        on:click=move |_| {
                            on_action.get_value().run(("delete_with_data".to_string(), hash.get_value()));
                        }
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="mr-2 h-4 w-4 opacity-70">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M20.25 7.5l-.625 10.632a2.25 2.25 0 01-2.247 2.118H6.622a2.25 2.25 0 01-2.247-2.118L3.75 7.5m6 4.125l2.25 2.25m0 0l2.25 2.25M12 13.875l2.25-2.25M12 13.875l-2.25-2.25M3.375 7.5h17.25c.621 0 1.125-.504 1.125-1.125v-1.5c0-.621-.504-1.125-1.125-1.125H3.375c-.621 0-1.125.504-1.125 1.125v1.5c0 .621.504 1.125 1.125 1.125z" />
                        </svg>
                        "Remove Data"
                    </ContextMenuItem>
                </ContextMenuContent>
            </Portal>
        </ContextMenu>
    }
}