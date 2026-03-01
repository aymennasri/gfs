//! `gfs providers` — list database providers and their supported versions (RFC 006).

use std::sync::Arc;

use anyhow::{Context, Result};
use gfs_domain::ports::database_provider::{
    DatabaseProviderRegistry, InMemoryDatabaseProviderRegistry, SupportedFeature,
};

use crate::output::{bold, cyan};

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

pub fn run(provider_name: Option<String>) -> Result<()> {
    let registry = Arc::new(InMemoryDatabaseProviderRegistry::new());
    gfs_compute_docker::containers::register_all(registry.as_ref())
        .context("failed to register database providers")?;

    match provider_name {
        Some(name) => print_provider_detail(registry.as_ref(), &name)?,
        None => print_all_providers(registry.as_ref())?,
    }
    Ok(())
}

fn print_all_providers(registry: &impl DatabaseProviderRegistry) -> Result<()> {
    let names = registry.list();
    if names.is_empty() {
        println!("  (no providers registered)");
        return Ok(());
    }

    let rows: Vec<_> = names
        .into_iter()
        .filter_map(|name| {
            let provider = registry.get(&name)?;
            let versions = provider.supported_versions().join(", ");
            let features = provider
                .supported_features()
                .iter()
                .map(|f| f.id.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            Some((name, versions, features))
        })
        .collect();

    print_providers_table(&rows);
    Ok(())
}

fn print_provider_detail(registry: &impl DatabaseProviderRegistry, name: &str) -> Result<()> {
    let provider = registry
        .get(name)
        .ok_or_else(|| anyhow::anyhow!("unknown provider: '{}'", name))?;

    let versions = provider.supported_versions();
    let features = provider.supported_features();

    println!("  {} {}", bold("Provider:"), cyan(name));
    println!();
    println!("  Supported versions: {}", versions.join(", "));
    println!();
    print_features_table(&features);
    println!();
    println!("  Images are pulled from Docker Hub by default.");
    Ok(())
}

// ---------------------------------------------------------------------------
// Output
// ---------------------------------------------------------------------------

/// Print providers table. Columns: database_provider | version | features
fn print_providers_table(rows: &[(String, String, String)]) {
    const COL_PROVIDER: usize = 20;
    const COL_VERSION: usize = 30;
    const COL_FEATURES: usize = 50;

    println!(
        "  {:<provider$} | {:<version$} | {:<features$}",
        bold("database_provider"),
        bold("version"),
        bold("features"),
        provider = COL_PROVIDER,
        version = COL_VERSION,
        features = COL_FEATURES
    );
    println!(
        "  {:<provider$}-+-{:<version$}-+-{:<features$}",
        "-".repeat(COL_PROVIDER),
        "-".repeat(COL_VERSION),
        "-".repeat(COL_FEATURES),
        provider = COL_PROVIDER,
        version = COL_VERSION,
        features = COL_FEATURES
    );

    for (name, versions, features) in rows {
        let version_trunc = truncate(versions, COL_VERSION);
        let features_trunc = truncate(features, COL_FEATURES);
        println!(
            "  {:<provider$} | {:<version$} | {:<features$}",
            cyan(name),
            version_trunc,
            features_trunc,
            provider = COL_PROVIDER,
            version = COL_VERSION,
            features = COL_FEATURES
        );
    }

    println!();
    println!("  Images are pulled from Docker Hub by default.");
}

/// Print features table. Columns: feature | description
fn print_features_table(features: &[SupportedFeature]) {
    const COL_FEATURE: usize = 25;
    const COL_DESC: usize = 55;

    println!("  {}", bold("Features"));
    println!("  {}", "─".repeat(COL_FEATURE + COL_DESC + 5));
    println!(
        "  {:<feature$} | {:<desc$}",
        bold("feature"),
        bold("description"),
        feature = COL_FEATURE,
        desc = COL_DESC
    );
    println!(
        "  {:<feature$}-+-{:<desc$}",
        "-".repeat(COL_FEATURE),
        "-".repeat(COL_DESC),
        feature = COL_FEATURE,
        desc = COL_DESC
    );

    for f in features {
        let desc_trunc = truncate(&f.description, COL_DESC);
        println!(
            "  {:<feature$} | {:<desc$}",
            f.id,
            desc_trunc,
            feature = COL_FEATURE,
            desc = COL_DESC
        );
    }
}

fn truncate(s: impl AsRef<str>, max_len: usize) -> String {
    let s = s.as_ref();
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}
