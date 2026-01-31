use leptos::*;
use thaw::*;
use shared::Torrent;

#[component]
pub fn TorrentTable(
    #[prop(into)] torrents: Signal<Vec<Torrent>>
) -> impl IntoView {
    view! {
        <div class="flex-1 overflow-auto bg-background">
            <table class="w-full text-left text-xs">
                <thead class="bg-muted/50 border-b border-border text-muted-foreground font-medium sticky top-0 bg-background z-10">
                    <tr>
                        <th class="px-2 py-1.5 font-medium">"Name"</th>
                        <th class="px-2 py-1.5 font-medium w-20 text-right">"Size"</th>
                        <th class="px-2 py-1.5 font-medium w-24">"Progress"</th>
                        <th class="px-2 py-1.5 font-medium w-20 text-center">"Status"</th>
                        <th class="px-2 py-1.5 font-medium w-20 text-right">"Down"</th>
                        <th class="px-2 py-1.5 font-medium w-20 text-right">"Up"</th>
                        <th class="px-2 py-1.5 font-medium w-20 text-right">"ETA"</th>
                    </tr>
                </thead>
                <tbody class="divide-y divide-border">
                    <For
                        each=move || torrents.get()
                        key=|t| t.hash.clone()
                        children=move |torrent| {
                            view! {
                                <tr class="hover:bg-muted/50 group transition-colors">
                                    <td class="px-2 py-1.5 truncate max-w-[200px]">{torrent.name}</td>
                                    <td class="px-2 py-1.5 text-right whitespace-nowrap text-muted-foreground">{torrent.size}</td>
                                    <td class="px-2 py-1.5">
                                        <Progress percentage=torrent.percent_complete as f32 />
                                    </td>
                                    <td class="px-2 py-1.5 text-center">
                                       <span class="text-[10px] px-1.5 py-0.5 rounded-full border border-border bg-background">
                                         {format!("{:?}", torrent.status)}
                                       </span>
                                    </td>
                                    <td class="px-2 py-1.5 text-right whitespace-nowrap text-blue-500">{torrent.down_rate}</td>
                                    <td class="px-2 py-1.5 text-right whitespace-nowrap text-green-500">{torrent.up_rate}</td>
                                    <td class="px-2 py-1.5 text-right whitespace-nowrap text-muted-foreground">{torrent.eta}</td>
                                </tr>
                            }
                        }
                    />
                </tbody>
            </table>
        </div>
    }
}
