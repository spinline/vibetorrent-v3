use leptos::wasm_bindgen::JsCast;
use leptos::prelude::*;
use leptos::logging;
use leptos::html;
use leptos::task::spawn_local;
use crate::api;

#[component]
pub fn Sidebar() -> impl IntoView {
    let store = use_context::<crate::store::TorrentStore>().expect("store not provided");

    let total_count = move || store.torrents.with(|map| map.len());
    let downloading_count = move || {
        store.torrents.with(|map| {
            map.values()
                .filter(|t| t.status == shared::TorrentStatus::Downloading)
                .count()
        })
    };
    let seeding_count = move || {
        store.torrents.with(|map| {
            map.values()
                .filter(|t| t.status == shared::TorrentStatus::Seeding)
                .count()
        })
    };
    let completed_count = move || {
        store.torrents.with(|map| {
            map.values()
                .filter(|t| {
                    t.status == shared::TorrentStatus::Seeding
                        || (t.status == shared::TorrentStatus::Paused && t.percent_complete >= 100.0)
                })
                .count()
        })
    };
    let paused_count = move || {
        store.torrents.with(|map| {
            map.values()
                .filter(|t| t.status == shared::TorrentStatus::Paused)
                .count()
        })
    };
    let inactive_count = move || {
        store.torrents.with(|map| {
            map.values()
                .filter(|t| {
                    t.status == shared::TorrentStatus::Paused
                        || t.status == shared::TorrentStatus::Error
                })
                .count()
        })
    };

    let close_drawer = move || {
        if let Some(element) = document().get_element_by_id("my-drawer") {
            if let Ok(input) = element.dyn_into::<web_sys::HtmlInputElement>() {
                input.set_checked(false);
            }
        }
    };

    let set_filter = move |f: crate::store::FilterStatus| {
        store.filter.set(f);
        close_drawer();
    };

    let filter_class = move |f: crate::store::FilterStatus| {
        if store.filter.get() == f {
            "active"
        } else {
            ""
        }
    };

    let handle_logout = move |_| {
        spawn_local(async move {
            if api::auth::logout().await.is_ok() {
                let _ = window().location().set_href("/login");
            }
        });
    };

                let username = move || {

                    store.user.get().unwrap_or_else(|| "User".to_string())

                };



                let first_letter = move || {

                    username().chars().next().unwrap_or('?').to_uppercase().to_string()

                };



                view! {

                    <div class="w-64 min-h-[100dvh] flex flex-col bg-base-200 border-r border-base-300 pb-8" style="padding-top: env(safe-area-inset-top);">

                        <div class="p-2 flex-1 overflow-y-auto">

                            <ul class="menu w-full rounded-box gap-1">

                                <li class="menu-title text-primary uppercase font-bold px-4">"Filters"</li>

                                <li>

                                    <button class={move || format!("cursor-pointer {}", filter_class(crate::store::FilterStatus::All))} on:click=move |_| set_filter(crate::store::FilterStatus::All)>

                                        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-5 h-5">

                                            <path stroke-linecap="round" stroke-linejoin="round" d="M3.75 6.75h16.5M3.75 12h16.5m-16.5 5.25h16.5" />

                                        </svg>

                                        "All"

                                        <span class="badge badge-sm badge-ghost ml-auto">{total_count}</span>

                                    </button>

                                </li>

                                <li>

                                    <button class={move || format!("cursor-pointer {}", filter_class(crate::store::FilterStatus::Downloading))} on:click=move |_| set_filter(crate::store::FilterStatus::Downloading)>

                                        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-5 h-5">

                                            <path stroke-linecap="round" stroke-linejoin="round" d="M3 16.5v2.25A2.25 2.25 0 005.25 21h13.5A2.25 2.25 0 0021 18.75V16.5M16.5 12L12 16.5m0 0L7.5 12m4.5 4.5V3" />

                                        </svg>

                                        "Downloading"

                                        <span class="badge badge-sm badge-ghost ml-auto">{downloading_count}</span>

                                    </button>

                                </li>

                                <li>

                                    <button class={move || format!("cursor-pointer {}", filter_class(crate::store::FilterStatus::Seeding))} on:click=move |_| set_filter(crate::store::FilterStatus::Seeding)>

                                        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-5 h-5">

                                            <path stroke-linecap="round" stroke-linejoin="round" d="M3 16.5v2.25A2.25 2.25 0 005.25 21h13.5A2.25 2.25 0 0021 18.75V16.5m-13.5-9L12 3m0 0l4.5 4.5M12 3v13.5" />

                                        </svg>

                                        "Seeding"

                                        <span class="badge badge-sm badge-ghost ml-auto">{seeding_count}</span>

                                    </button>

                                </li>

                                <li>

                                    <button class={move || format!("cursor-pointer {}", filter_class(crate::store::FilterStatus::Completed))} on:click=move |_| set_filter(crate::store::FilterStatus::Completed)>

                                        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-5 h-5">

                                            <path stroke-linecap="round" stroke-linejoin="round" d="M9 12.75L11.25 15 15 9.75M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />

                                        </svg>

                                        "Completed"

                                        <span class="badge badge-sm badge-ghost ml-auto">{completed_count}</span>

                                    </button>

                                </li>

                                <li>

                                    <button class={move || format!("cursor-pointer {}", filter_class(crate::store::FilterStatus::Paused))} on:click=move |_| set_filter(crate::store::FilterStatus::Paused)>

                                        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-5 h-5">

                                            <path stroke-linecap="round" stroke-linejoin="round" d="M15.75 5.25v13.5m-7.5-13.5v13.5" />

                                        </svg>

                                        "Paused"

                                        <span class="badge badge-sm badge-ghost ml-auto">{paused_count}</span>

                                    </button>

                                </li>

                                <li>

                                    <button class={move || format!("cursor-pointer {}", filter_class(crate::store::FilterStatus::Inactive))} on:click=move |_| set_filter(crate::store::FilterStatus::Inactive)>

                                        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-5 h-5">

                                            <path stroke-linecap="round" stroke-linejoin="round" d="M18.364 18.364A9 9 0 005.636 5.636m12.728 12.728A9 9 0 015.636 5.636m12.728 12.728L5.636 5.636" />

                                        </svg>

                                        "Inactive"

                                         <span class="badge badge-sm badge-ghost ml-auto">{inactive_count}</span>

                                    </button>

                                </li>

                            </ul>

                        </div>



                        <div class="p-4 border-t border-base-300 bg-base-200/50">

                            <div class="flex items-center gap-3">

                                <div class="avatar">

                                    <div class="w-8 rounded-full bg-neutral text-neutral-content ring ring-primary ring-offset-base-100 ring-offset-1">

                                        <span class="text-sm font-bold flex items-center justify-center h-full">{first_letter}</span>

                                    </div>

                                </div>

                                <div class="flex-1 overflow-hidden">

                                    <div class="font-bold text-sm truncate">{username}</div>

                                    <div class="text-[10px] text-base-content/60 truncate">"Online"</div>

                                </div>

                                <button

                                    class="btn btn-ghost btn-xs btn-square text-error hover:bg-error/10"

                                    title="Logout"

                                    on:click=handle_logout

                                >

                                    <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-4 h-4">

                                        <path stroke-linecap="round" stroke-linejoin="round" d="M15.75 9V5.25A2.25 2.25 0 0013.5 3h-6a2.25 2.25 0 00-2.25 2.25v13.5A2.25 2.25 0 007.5 21h6a2.25 2.25 0 002.25-2.25V15M12 9l-3 3m0 0l3 3m-3-3h12.75" />

                                    </svg>

                                </button>

                            </div>

                        </div>

                    </div>

                }}
