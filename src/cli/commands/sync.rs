//! Sync commands (push, pull, fetch, remote)

use pkm_ai::versioning::sync::{MergeStrategy, Remote, RemoteConfig, SyncEngine};
use anyhow::Context;
use tracing::info;
use std::path::Path;

/// Execute a push command
pub async fn push(
    repo_path: Option<&str>,
    remote_name: &str,
    refs: &[String],
    force: bool,
) -> anyhow::Result<()> {
    let repo = resolve_repo(repo_path)?;
    let engine = SyncEngine::new(&repo).context("Failed to create sync engine")?;

    let remote = Remote::new(remote_name, &repo, remote_name);

    if !remote.exists() {
        anyhow::bail!("Remote '{}' does not exist at path '{}'", remote_name, remote_name);
    }

    let ref_specs: Vec<&str> = if refs.is_empty() {
        vec!["main", "master"]
    } else {
        refs.iter().map(|s| s.as_str()).collect()
    };

    info!("Pushing to remote '{}' at {:?}", remote_name, remote.path());

    let result = engine.push(&remote, &ref_specs, force).context("Push failed")?;

    if result.success {
        info!("Push completed successfully");
        for update in &result.updated_refs {
            let ff_status = if update.fast_forward {
                "fast-forward"
            } else {
                "forced"
            };
            println!("  {}: {} -> {} [{}]",
                update.ref_name,
                update.old_value.map(|u| u.to_string()).unwrap_or_else(|| "none".to_string()),
                update.new_value.map(|u| u.to_string()).unwrap_or_else(|| "none".to_string()),
                ff_status
            );
        }
    }

    Ok(())
}

/// Execute a pull command
pub async fn pull(
    repo_path: Option<&str>,
    remote_name: &str,
    branch: &str,
    strategy: Option<&str>,
) -> anyhow::Result<()> {
    let repo = resolve_repo(repo_path)?;
    let engine = SyncEngine::new(&repo).context("Failed to create sync engine")?;

    let remote = Remote::new(remote_name, &repo, remote_name);

    if !remote.exists() {
        anyhow::bail!("Remote '{}' does not exist at path '{}'", remote_name, remote_name);
    }

    let merge_strategy = match strategy.unwrap_or("merge") {
        "ours" => MergeStrategy::Ours,
        "theirs" => MergeStrategy::Theirs,
        _ => MergeStrategy::Merge,
    };

    info!("Pulling from remote '{}' branch '{}'", remote_name, branch);

    let result = engine.pull(&remote, branch, merge_strategy).await.context("Pull failed")?;

    if result.success {
        info!("Pull completed successfully");
        for update in &result.updated_refs {
            println!("  {}: {} -> {}",
                update.ref_name,
                update.old_value.map(|u| u.to_string()).unwrap_or_else(|| "none".to_string()),
                update.new_value.map(|u| u.to_string()).unwrap_or_else(|| "none".to_string())
            );
        }
    }

    Ok(())
}

/// Execute a fetch command
pub async fn fetch(
    repo_path: Option<&str>,
    remote_name: &str,
) -> anyhow::Result<()> {
    let repo = resolve_repo(repo_path)?;
    let engine = SyncEngine::new(&repo).context("Failed to create sync engine")?;

    let remote = Remote::new(remote_name, &repo, remote_name);

    if !remote.exists() {
        anyhow::bail!("Remote '{}' does not exist at path '{}'", remote_name, remote_name);
    }

    info!("Fetching from remote '{}'", remote_name);

    let result = engine.fetch(&remote).context("Fetch failed")?;

    if result.success {
        info!("Fetch completed successfully");
        println!("Updated refs:");
        for update in &result.updated_refs {
            println!("  {}: {} -> {}",
                update.ref_name,
                update.old_value.map(|u| u.to_string()).unwrap_or_else(|| "none".to_string()),
                update.new_value.map(|u| u.to_string()).unwrap_or_else(|| "none".to_string())
            );
        }
    }

    Ok(())
}

/// List remotes
pub async fn remote_list(repo_path: Option<&str>) -> anyhow::Result<()> {
    let repo = resolve_repo(repo_path)?;
    ensure_pkm_structure(&repo)?;
    let remotes_path = repo.join(".pkm").join("remotes");

    if !remotes_path.exists() {
        println!("No remotes configured");
        return Ok(());
    }

    for entry in std::fs::read_dir(&remotes_path)? {
        let entry = entry?;
        if entry.file_type()?.is_dir()
            && let Some(name) = entry.file_name().to_str() {
                let config_path = entry.path().join("config");
                if let Ok(config_content) = std::fs::read_to_string(&config_path) {
                    println!("{}\t{}", name, config_content.trim());
                } else {
                    println!("{}", name);
                }
            }
    }

    Ok(())
}

/// Add a remote
pub async fn remote_add(
    repo_path: Option<&str>,
    name: &str,
    url: &str,
) -> anyhow::Result<()> {
    let repo = resolve_repo(repo_path)?;
    ensure_pkm_structure(&repo)?;
    let remotes_path = repo.join(".pkm").join("remotes");
    let remote_path = remotes_path.join(name);

    std::fs::create_dir_all(&remote_path).context("Failed to create remote directory")?;

    let config = RemoteConfig::new(name, url);
    let config_path = remote_path.join("config");
    let config_content = serde_json::to_string_pretty(&config).context("Failed to serialize config")?;
    std::fs::write(&config_path, config_content).context("Failed to write remote config")?;

    // Initialize the remote repository structure
    let remote_pkm = remote_path.join(".pkm");
    std::fs::create_dir_all(remote_pkm.join("objects"))?;
    std::fs::create_dir_all(remote_pkm.join("refs").join("heads"))?;
    std::fs::create_dir_all(remote_pkm.join("refs").join("tags"))?;

    info!("Remote '{}' added at '{}'", name, url);
    println!("Remote '{}' added successfully", name);

    Ok(())
}

/// Clone a repository from a source path to a destination path
pub async fn clone(
    source: &str,
    destination: Option<&str>,
    _branch: Option<&str>,
    depth: Option<usize>,
) -> anyhow::Result<()> {
    let source_path = Path::new(source);

    if !source_path.exists() {
        anyhow::bail!("Source path '{}' does not exist", source);
    }

    // Check if source is a valid PKM repository
    let source_pkm = source_path.join(".pkm");
    if !source_pkm.exists() {
        anyhow::bail!("Source path '{}' is not a PKM repository (missing .pkm directory)", source);
    }

    // Determine destination path
    let dest_path = if let Some(dest) = destination {
        Path::new(dest).to_path_buf()
    } else {
        // Use the source directory name as destination
        source_path.file_name()
            .map(|n| Path::new(".").join(n))
            .unwrap_or_else(|| Path::new(".").to_path_buf())
    };

    if dest_path.exists() {
        anyhow::bail!("Destination path '{}' already exists", dest_path.display());
    }

    info!("Cloning from '{}' to '{}'", source, dest_path.display());

    // Create destination directory structure
    std::fs::create_dir_all(&dest_path).context("Failed to create destination directory")?;
    let dest_pkm = dest_path.join(".pkm");
    std::fs::create_dir_all(dest_pkm.join("objects"))?;
    std::fs::create_dir_all(dest_pkm.join("refs").join("heads"))?;
    std::fs::create_dir_all(dest_pkm.join("refs").join("tags"))?;
    std::fs::create_dir_all(dest_pkm.join("remotes"))?;

    // Copy objects from source to destination
    let source_objects = source_pkm.join("objects");
    let dest_objects = dest_pkm.join("objects");

    if source_objects.exists() {
        copy_directory_recursive(&source_objects, &dest_objects)
            .context("Failed to copy objects")?;
    }

    // Copy refs from source to destination
    let source_refs = source_pkm.join("refs");
    let dest_refs = dest_pkm.join("refs");

    if source_refs.exists() {
        copy_directory_recursive(&source_refs, &dest_refs)
            .context("Failed to copy refs")?;
    }

    // Shallow clone if depth is specified
    if let Some(d) = depth {
        let shallow_info = ShallowCloneInfo {
            depth: Some(d),
            ..Default::default()
        };
        let shallow_path = dest_pkm.join("shallow");
        let shallow_content = serde_json::to_string(&shallow_info)
            .context("Failed to serialize shallow info")?;
        std::fs::write(&shallow_path, shallow_content)
            .context("Failed to write shallow file")?;
    }

    println!("Cloned successfully to '{}'", dest_path.display());
    println!("Note: Working set and branch state may need initialization");

    Ok(())
}

/// Copy directory recursively
fn copy_directory_recursive(src: &Path, dst: &Path) -> anyhow::Result<()> {
    if !src.is_dir() {
        return Ok(());
    }

    std::fs::create_dir_all(dst)?;

    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_dir() {
            copy_directory_recursive(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)
                .context(format!("Failed to copy '{}'", src_path.display()))?;
        }
    }

    Ok(())
}

/// Shallow clone information
#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
struct ShallowCloneInfo {
    depth: Option<usize>,
    #[serde(default)]
    shallow_commits: Vec<String>,
}

/// Resolve the repository path
fn resolve_repo(repo_path: Option<&str>) -> anyhow::Result<std::path::PathBuf> {
    match repo_path {
        Some(path) => {
            let path = Path::new(path);
            if !path.exists() {
                anyhow::bail!("Repository path '{}' does not exist", path.display());
            }
            Ok(path.to_path_buf())
        }
        None => std::env::current_dir().context("Failed to get current directory"),
    }
}

/// Initialize .pkm structure if it doesn't exist
fn ensure_pkm_structure(repo: &std::path::Path) -> anyhow::Result<()> {
    let pkm_path = repo.join(".pkm");
    if !pkm_path.exists() {
        std::fs::create_dir_all(pkm_path.join("objects"))?;
        std::fs::create_dir_all(pkm_path.join("refs").join("heads"))?;
        std::fs::create_dir_all(pkm_path.join("refs").join("tags"))?;
        std::fs::create_dir_all(pkm_path.join("remotes"))?;
    }
    Ok(())
}
