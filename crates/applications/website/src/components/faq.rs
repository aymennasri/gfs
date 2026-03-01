use leptos::*;

#[derive(Clone)]
struct FaqItem {
    question: &'static str,
    answer: &'static str,
}

#[component]
pub fn Faq() -> impl IntoView {
    let faqs = vec![
        FaqItem {
            question: "What databases does GFS support?",
            answer: "GFS currently supports PostgreSQL (versions 13-18) and MySQL (versions 8.0-8.1).",
        },
        FaqItem {
            question: "Is GFS ready for production?",
            answer: "GFS is for local use only. If you need a production ready database versioning system, check https://app.guepard.run",
        },
        FaqItem {
            question: "How does GFS work?",
            answer: "GFS uses Docker to manage isolated database environments and creates snapshots of your database state at each commit, allowing you to travel through history and work with branches just like Git.",
        },
        FaqItem {
            question: "Do I need Docker?",
            answer: "Yes, Docker is required to run GFS as it manages database containers for isolation and versioning.",
        },
        FaqItem {
            question: "Can I use GFS with my existing database?",
            answer: "GFS creates and manages its own database instances. You can import data from existing databases into a GFS-managed repository.",
        },
        FaqItem {
            question: "Is GFS open source?",
            answer: "Yes, GFS is licensed under the Elastic License v2 (ELv2) and the source code is available on GitHub.",
        },
    ];

    let (expanded, set_expanded) = create_signal::<Option<usize>>(None);

    view! {
        <section class="faq-section">
            <div class="container">
                <h2 class="section-title">"Frequently Asked Questions"</h2>
                <div class="faq-list">
                    {faqs.into_iter().enumerate().map(|(idx, faq)| {
                        view! {
                            <div class="faq-item">
                                <button
                                    class="faq-question"
                                    on:click=move |_| {
                                        if expanded.get() == Some(idx) {
                                            set_expanded.set(None)
                                        } else {
                                            set_expanded.set(Some(idx))
                                        }
                                    }
                                >
                                    <span>{faq.question}</span>
                                    <span class="faq-icon">
                                        {move || if expanded.get() == Some(idx) { "−" } else { "+" }}
                                    </span>
                                </button>
                                <div class=move || {
                                    if expanded.get() == Some(idx) {
                                        "faq-answer expanded"
                                    } else {
                                        "faq-answer"
                                    }
                                }>
                                    <p>{faq.answer}</p>
                                </div>
                            </div>
                        }
                    }).collect::<Vec<_>>()}
                </div>
            </div>
        </section>
    }
}
