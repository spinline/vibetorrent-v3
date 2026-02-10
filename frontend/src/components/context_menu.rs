use leptos::prelude::*;
use web_sys::MouseEvent;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

// ── Kendi reaktif Context Menu implementasyonumuz ──
// leptos-shadcn-context-menu v0.8.1'de ContextMenuContent'te
// `if open.get()` statik kontrolü reaktif değil. Aşağıda
// `Show` bileşeni ile düzgün reaktif versiyon yer alıyor.

#[component]
pub fn TorrentContextMenu(
    children: Children,
    torrent_hash: String,
    on_action: Callback<(String, String)>,
) -> impl IntoView {
    let hash = StoredValue::new(torrent_hash);
    let on_action = StoredValue::new(on_action);

    let open = RwSignal::new(false);
    let position = RwSignal::new((0i32, 0i32));

    // Sağ tıklama handler
    let on_contextmenu = move |e: MouseEvent| {
        e.prevent_default();
        e.stop_propagation();
        position.set((e.client_x(), e.client_y()));
        open.set(true);
    };

    // Menü dışına tıklandığında kapanma
    Effect::new(move |_| {
        if open.get() {
            let cb = Closure::wrap(Box::new(move |_: MouseEvent| {
                open.set(false);
            }) as Box<dyn Fn(MouseEvent)>);

            let window = web_sys::window().unwrap();
            let document = window.document().unwrap();
            let _ = document.add_event_listener_with_callback(
                "click",
                cb.as_ref().unchecked_ref(),
            );

            // Cleanup: tek sefer dinleyici — click yakalandığında otomatik kapanıp listener kalıyor
            // ama open=false olduğunda effect tekrar çalışmaz, böylece sorun yok.
            cb.forget();
        }
    });

    let menu_action = move |action: &'static str| {
        open.set(false);
        on_action.get_value().run((action.to_string(), hash.get_value()));
    };

    view! {
        <div
            class="w-full"
            on:contextmenu=on_contextmenu
        >
            {children()}
        </div>

        <Show when=move || open.get()>
            {
                    let (x, y) = position.get();
                    // Menü yaklaşık boyutları
                    let menu_width = 200;
                    let menu_height = 220;
                    let window = web_sys::window().unwrap();
                    let vw = window.inner_width().unwrap().as_f64().unwrap() as i32;
                    let vh = window.inner_height().unwrap().as_f64().unwrap() as i32;
                    // Sağa taşarsa sola aç, alta taşarsa yukarı aç
                    let final_x = if x + menu_width > vw { x - menu_width } else { x };
                    let final_y = if y + menu_height > vh { y - menu_height } else { y };
                    let final_x = final_x.max(0);
                    let final_y = final_y.max(0);
                    view! {
                        <div
                            class="fixed inset-0 z-[99]"
                            on:click=move |e: MouseEvent| {
                                e.stop_propagation();
                                open.set(false);
                            }
                            on:contextmenu=move |e: MouseEvent| {
                                e.prevent_default();
                                e.stop_propagation();
                                open.set(false);
                            }
                        />
                        <div
                            class="fixed z-[100] min-w-[12rem] overflow-hidden rounded-md border bg-popover p-1 text-popover-foreground shadow-md animate-in fade-in-0 zoom-in-95"
                            style=format!("left: {}px; top: {}px;", final_x, final_y)
                            on:click=move |e: MouseEvent| e.stop_propagation()
                    >
                        // Start
                        <div
                            class="relative flex cursor-pointer select-none items-center rounded-sm px-2 py-1.5 text-sm outline-none transition-colors hover:bg-accent hover:text-accent-foreground"
                            on:click=move |_| menu_action("start")
                        >
                            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="mr-2 h-4 w-4 opacity-70">
                                <path stroke-linecap="round" stroke-linejoin="round" d="M5.25 5.653c0-.856.917-1.398 1.667-.986l11.54 6.348a1.125 1.125 0 010 1.971l-11.54 6.347a1.125 1.125 0 01-1.667-.985V5.653z" />
                            </svg>
                            "Start"
                        </div>

                        // Stop
                        <div
                            class="relative flex cursor-pointer select-none items-center rounded-sm px-2 py-1.5 text-sm outline-none transition-colors hover:bg-accent hover:text-accent-foreground"
                            on:click=move |_| menu_action("stop")
                        >
                            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="mr-2 h-4 w-4 opacity-70">
                                <path stroke-linecap="round" stroke-linejoin="round" d="M15.75 5.25v13.5m-7.5-13.5v13.5" />
                            </svg>
                            "Stop"
                        </div>

                        // Recheck
                        <div
                            class="relative flex cursor-pointer select-none items-center rounded-sm px-2 py-1.5 text-sm outline-none transition-colors hover:bg-accent hover:text-accent-foreground"
                            on:click=move |_| menu_action("recheck")
                        >
                            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="mr-2 h-4 w-4 opacity-70">
                                <path stroke-linecap="round" stroke-linejoin="round" d="M16.023 9.348h4.992v-.001M2.985 19.644v-4.992m0 0h4.992m-4.993 0l3.181 3.183a8.25 8.25 0 0013.803-3.7M4.031 9.865a8.25 8.25 0 0113.803-3.7l3.181 3.182m0-4.991v4.99" />
                            </svg>
                            "Recheck"
                        </div>

                        // Separator
                        <div class="-mx-1 my-1 h-px bg-border" />

                        // Remove
                        <div
                            class="relative flex cursor-pointer select-none items-center rounded-sm px-2 py-1.5 text-sm outline-none transition-colors text-destructive hover:bg-destructive hover:text-destructive-foreground"
                            on:click=move |_| menu_action("delete")
                        >
                            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="mr-2 h-4 w-4 opacity-70">
                                <path stroke-linecap="round" stroke-linejoin="round" d="M14.74 9l-.346 9m-4.788 0L9.26 9m9.968-3.21c.342.052.682.107 1.022.166m-1.022-.165L18.16 19.673a2.25 2.25 0 01-2.244 2.077H8.084a2.25 2.25 0 01-2.244-2.077L4.772 5.79m14.456 0a48.108 48.108 0 00-3.478-.397m-12 .562c.34-.059.68-.114 1.022-.165m0 0a48.11 48.11 0 013.478-.397m7.5 0v-.916c0-1.18-.91-2.164-2.09-2.164h-2.34c-1.18 0-2.09.984-2.09 2.164v.916m7.5 0a48.667 48.667 0 00-7.5 0" />
                            </svg>
                            "Remove"
                        </div>

                        // Remove with Data
                        <div
                            class="relative flex cursor-pointer select-none items-center rounded-sm px-2 py-1.5 text-sm outline-none transition-colors text-destructive hover:bg-destructive hover:text-destructive-foreground"
                            on:click=move |_| menu_action("delete_with_data")
                        >
                            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="mr-2 h-4 w-4 opacity-70">
                                <path stroke-linecap="round" stroke-linejoin="round" d="M20.25 7.5l-.625 10.632a2.25 2.25 0 01-2.247 2.118H6.622a2.25 2.25 0 01-2.247-2.118L3.75 7.5m6 4.125l2.25 2.25m0 0l2.25 2.25M12 13.875l2.25-2.25M12 13.875l-2.25-2.25M3.375 7.5h17.25c.621 0 1.125-.504 1.125-1.125v-1.5c0-.621-.504-1.125-1.125-1.125H3.375c-.621 0-1.125.504-1.125 1.125v1.5c0 .621.504 1.125 1.125 1.125z" />
                            </svg>
                            "Remove with Data"
                        </div>
                    </div>
                }
            }
        </Show>
    }
}