use leptos::*;

#[component]
pub fn Hero() -> impl IntoView {
    view! {
        <section class="hero">
            <div class="container">
                <div class="hero-content">
                    <h1 class="hero-title">
                        <span class="highlight">"Git-like version control"</span>
                        " for your databases and "
                        <a href="#video-showcase" class="hero-more-link">"more"</a>
                        " 🚀"
                    </h1>
                    <p class="hero-subtitle">
                        "Supercharge your AI agents with database branching, version control, and easier operations."
                    </p>
                    <div class="hero-badges-wrapper">
                        <p class="hero-badges-label">"Fully compatible with"</p>
                        <div class="hero-badges">
                        <a href="/docs/mcp/cursor" target="_blank" rel="noopener noreferrer" class="hero-badge" title="Cursor">
                            <img src="/public/assets/cursor-logo.png" alt="Cursor" class="hero-badge-img"/>
                        </a>
                        <a href="/docs/mcp/claude-code" target="_blank" rel="noopener noreferrer" class="hero-badge" title="Claude Code">
                            <img src="/public/assets/claude-logo.svg" alt="Claude Code" class="hero-badge-img"/>
                        </a>
                        <a href="/docs/installation" target="_blank" rel="noopener noreferrer" class="hero-badge" title="PostgreSQL">
                            <img src="/public/assets/postgresql.svg" alt="PostgreSQL" class="hero-badge-img"/>
                        </a>
                        <a href="/docs/installation" target="_blank" rel="noopener noreferrer" class="hero-badge" title="MySQL">
                            <img src="/public/assets/mysql.svg" alt="MySQL" class="hero-badge-img"/>
                        </a>
                        </div>
                    </div>
                </div>
            </div>
        </section>
    }
}
