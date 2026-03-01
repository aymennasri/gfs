use leptos::*;

/// A use case to showcase with its own video.
#[derive(Clone)]
pub struct UseCase {
    pub tab_label: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub video_src: &'static str,
}

const USE_CASES: &[UseCase] = &[
    UseCase {
        tab_label: "GFS in Action",
        title: "Git for databases in Action",
        description: "Commit, branch, merge, and time-travel your database state—just like Git.",
        video_src: "/public/assets/gfs-showcase.mp4",
    },
    UseCase {
        tab_label: "Database Ops",
        title: "Easier database ops for Agents",
        description: "Let AI agents easily interact with your database. Export and import your database state with ease. Compare schemas and more.",
        video_src: "/public/assets/gfs-ops-showcase.mp4",
    },
    UseCase {
        tab_label: "Expert Subagents",
        title: "Expert subagents for database queries",
        description: "Specialized subagents that query and evolve your database with full version control.",
        video_src: "/public/assets/gfs-claude-code.mp4",
    },
];

#[component]
pub fn VideoShowcase() -> impl IntoView {
    let (active_index, set_active_index) = create_signal(0usize);

    view! {
        <section id="video-showcase" class="video-section">
            <div class="container">
                <h2 class="section-title">"See GFS in Action"</h2>
                <div class="video-showcase-tabs">
                    <div class="tabs-header">
                        {USE_CASES.iter().enumerate().map(|(i, uc)| {
                            view! {
                                <button
                                    class=move || if active_index.get() == i { "tab active" } else { "tab" }
                                    on:click=move |_| set_active_index.set(i)
                                >
                                    {uc.tab_label}
                                </button>
                            }
                        }).collect_view()}
                    </div>
                    <div class="video-showcase-content">
                        {move || {
                            let uc = &USE_CASES[active_index.get().min(USE_CASES.len() - 1)];
                            view! {
                                <div class="video-showcase-panel">
                                    <h3 class="video-showcase-title">{uc.title}</h3>
                                    <p class="video-showcase-description">{uc.description}</p>
                                    <div class="video-container">
                                        <video
                                            autoplay=true
                                            muted=true
                                            loop=true
                                            playsinline=true
                                            controls=true
                                            class="showcase-video"
                                        >
                                            <source src=uc.video_src type="video/mp4"/>
                                            "Your browser does not support the video tag."
                                        </video>
                                    </div>
                                </div>
                            }
                        }}
                    </div>
                </div>
            </div>
        </section>
    }
}
