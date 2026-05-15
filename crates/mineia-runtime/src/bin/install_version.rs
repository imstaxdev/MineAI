use mineia_core::MineiaContext;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let version = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "1.21.8".to_owned());
    let context = MineiaContext::open_default()?;
    let report = mineia_runtime::install_vanilla_version(&context.paths, &version).await?;

    println!("installed={}", report.version);
    println!("downloaded_files={}", report.downloaded_files);
    println!("reused_files={}", report.reused_files);
    println!("libraries={}", report.libraries);
    println!("assets={}", report.assets);
    println!("natives={}", report.natives);
    println!("version_dir={}", report.version_dir.display());

    Ok(())
}
