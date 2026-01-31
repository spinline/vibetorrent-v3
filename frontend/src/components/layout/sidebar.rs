use leptos::*;

#[component]
pub fn Sidebar() -> impl IntoView {
    view! {
        <aside class="w-64 bg-base-200 h-full flex flex-col border-r border-base-300">
            <div class="p-4">
                <h2 class="text-xl font-bold px-4 mb-2 text-primary">"Filters"</h2>
                <ul class="menu w-full rounded-box gap-1">
                    <li>
                        <a class="active">
                            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-5 h-5">
                                <path stroke-linecap="round" stroke-linejoin="round" d="M3.75 6.75h16.5M3.75 12h16.5m-16.5 5.25h16.5" />
                            </svg>
                            "All"
                            <span class="badge badge-sm badge-ghost ml-auto">"12"</span>
                        </a>
                    </li>
                    <li>
                        <a>
                            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-5 h-5">
                                <path stroke-linecap="round" stroke-linejoin="round" d="M3 16.5v2.25A2.25 2.25 0 005.25 21h13.5A2.25 2.25 0 0021 18.75V16.5M16.5 12L12 16.5m0 0L7.5 12m4.5 4.5V3" />
                            </svg>
                            "Downloading"
                            <span class="badge badge-sm badge-ghost ml-auto">"4"</span>
                        </a>
                    </li>
                    <li>
                        <a>
                            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-5 h-5">
                                <path stroke-linecap="round" stroke-linejoin="round" d="M3 16.5v2.25A2.25 2.25 0 005.25 21h13.5A2.25 2.25 0 0021 18.75V16.5m-13.5-9L12 3m0 0l4.5 4.5M12 3v13.5" />
                            </svg>
                            "Seeding"
                            <span class="badge badge-sm badge-ghost ml-auto">"8"</span>
                        </a>
                    </li>
                    <li>
                        <a>
                            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-5 h-5">
                                <path stroke-linecap="round" stroke-linejoin="round" d="M9 12.75L11.25 15 15 9.75M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                            </svg>
                            "Completed"
                        </a>
                    </li>
                    <li>
                        <a>
                            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-5 h-5">
                                <path stroke-linecap="round" stroke-linejoin="round" d="M18.364 18.364A9 9 0 005.636 5.636m12.728 12.728A9 9 0 015.636 5.636m12.728 12.728L5.636 5.636" />
                            </svg>
                            "Inactive"
                        </a>
                    </li>
                </ul>
            </div>
            
            <div class="mt-auto p-4 border-t border-base-300">
                <h3 class="text-xs font-bold text-base-content/50 uppercase mb-2 px-4">"Trackers"</h3>
                 <ul class="menu w-full rounded-box gap-1 text-sm">
                    <li><a>"All Trackers"</a></li>
                    <li><a>"Error"</a></li>
                </ul>
            </div>
        </aside>
    }
}
