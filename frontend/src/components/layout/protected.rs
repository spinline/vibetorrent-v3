use leptos::prelude::*;
use crate::components::layout::sidebar::Sidebar;
use crate::components::layout::toolbar::Toolbar;
use crate::components::layout::statusbar::StatusBar;
use crate::components::ui::sidenav::{SidenavWrapper, Sidenav, SidenavInset};

#[component]
pub fn Protected(children: Children) -> impl IntoView {
    view! {
        <SidenavWrapper attr:style="--sidenav-width:16rem; --sidenav-width-icon:3rem;">
            // Masaüstü Sidenav
            <Sidenav>
                <Sidebar />
            </Sidenav>

            // İçerik Alanı
            <SidenavInset class="flex flex-col h-screen overflow-hidden">
                // Toolbar (Üst Bar)
                <Toolbar />
                
                // Ana İçerik
                <main class="flex-1 overflow-hidden relative bg-background">
                    {children()}
                </main>

                // Alt Bar
                <StatusBar />
            </SidenavInset>
        </SidenavWrapper>
    }
}
