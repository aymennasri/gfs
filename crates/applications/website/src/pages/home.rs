use crate::components::{CodeTabs, Faq, FeatureCard, Hero, VideoShowcase};
use leptos::*;

#[component]
pub fn Home() -> impl IntoView {
    view! {
        <div class="home">
            <Hero/>
            <CodeTabs/>
            <VideoShowcase/>

            <section class="features-section">
                <div class="container">
                    <h2 class="section-title">"Why GFS?"</h2>
                    <div class="features-grid">
                        <FeatureCard
                            icon="🧪"
                            title="Experiment with Confidence"
                            description="Let AI agents experiment freely on databases—rollback anytime. No more fear of breaking things."
                        />
                        <FeatureCard
                            icon="⚡"
                            title="More Efficient Agents"
                            description="Prebuilt tools, skills, subagents, and ops for a better Agentic Experience (AX)."
                        />
                        <FeatureCard
                            icon="📦"
                            title="Full Isolation"
                            description="Runs on containers, microVMs, or processes, your database experiments stay isolated and safe."
                        />
                        <FeatureCard
                            icon="🤝"
                            title="Easy for Humans & Agents"
                            description="Git-like semantics for databases. No new syntax to learn, works naturally for both humans and AI."
                        />
                    </div>
                </div>
            </section>

            <section class="workflow-section">
                <div class="container">
                    <h2 class="section-title">"Simple Workflow"</h2>
                    <div class="workflow-steps">
                        <div class="workflow-step">
                            <div class="step-number">"1"</div>
                            <h3>"Initialize"</h3>
                            <pre><code>"gfs init --database-provider postgres --database-version 17"</code></pre>
                        </div>
                        <div class="workflow-step">
                            <div class="step-number">"2"</div>
                            <h3>"Make Changes"</h3>
                            <pre><code>"gfs query \"SELECT * FROM users LIMIT 3\""</code></pre>
                        </div>
                        <div class="workflow-step">
                            <div class="step-number">"3"</div>
                            <h3>"Commit"</h3>
                            <pre><code>"gfs commit -m \"Add user table\""</code></pre>
                        </div>
                        <div class="workflow-step">
                            <div class="step-number">"4"</div>
                            <h3>"Time Travel"</h3>
                            <pre><code>"gfs log → gfs checkout HEAD~1"</code></pre>
                        </div>
                    </div>
                </div>
            </section>

            <section class="cta-section">
                <div class="container">
                    <div class="cta-box">
                        <h2>"Ready to get started?"</h2>
                        <p>"Join the community and help shape the future of database version control."</p>
                        <div class="cta-buttons">
                            <a href="/docs" class="btn-primary">"Read the Docs"</a>
                            <a href="https://github.com/Guepard-Corp/gfs" target="_blank" class="btn-secondary">"View on GitHub"</a>
                        </div>
                    </div>
                </div>
            </section>

            <Faq/>
        </div>
    }
}
