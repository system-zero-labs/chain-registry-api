use std::fs;
use std::path;

// TODO: vectors are PathBuf so the caller can use the paths directly
pub struct ChainDirs {
    mainnets: Vec<String>,
    testnets: Vec<String>,
}

pub fn shallow_clone(
    remote: String,
    git_ref: String,
    clone_dir: path::PathBuf,
) -> anyhow::Result<ChainDirs> {
    let mut cmd = std::process::Command::new("git");
    cmd.arg("clone")
        .arg("--depth")
        .arg("1")
        .arg("--branch")
        .arg(git_ref)
        .arg(remote)
        .arg(clone_dir.to_str().unwrap());
    let output = cmd.output()?;
    if !output.status.success() {
        anyhow::bail!(
            "git clone failed with status: {:?} stderr: {:?} stdout: {:?}",
            output.status,
            std::str::from_utf8(output.stderr.as_ref()).unwrap_or("cannot read stderr"),
            std::str::from_utf8(output.stdout.as_ref()).unwrap_or("cannot read stdout")
        );
    }

    let mainnets = fs::read_dir(clone_dir.clone())?
        .filter_map(|f| {
            let f = f.unwrap().path();
            if !f.is_dir() {
                return None;
            }
            match f.file_name() {
                Some(f) => {
                    let name = f.to_str().unwrap();
                    if name.starts_with("_") || name.starts_with("testnets") {
                        return None;
                    }
                    return Some(name.to_string());
                }
                _ => None,
            }
        })
        .collect();

    let testnets = fs::read_dir(clone_dir.join("testnets"))?
        .filter_map(|f| {
            let f = f.unwrap().path();
            if !f.is_dir() {
                return None;
            }
            match f.file_name() {
                Some(f) => {
                    let name = f.to_str().unwrap();
                    if name.starts_with("_") {
                        return None;
                    }
                    return Some(name.to_string());
                }
                _ => None,
            }
        })
        .collect();

    Ok(ChainDirs { mainnets, testnets })
}

#[cfg(test)]
mod tests {
    use crate::hydrate::shallow_clone;
    use tempfile::TempDir;

    #[test]
    fn test_shallow_clone() {
        let temp_dir = TempDir::new().unwrap();

        let chains = shallow_clone(
            "https://github.com/cosmos/chain-registry".to_string(),
            "master".to_string(),
            temp_dir.path().to_path_buf(),
        )
        .unwrap();

        assert!(temp_dir.path().join("cosmoshub/chain.json").exists());

        assert!(chains.mainnets.len() > 1);
        assert!(
            chains.mainnets.contains(&"cosmoshub".to_string()),
            "{:?}",
            chains.mainnets
        );
        assert!(
            chains.mainnets.contains(&"osmosis".to_string()),
            "{:?}",
            chains.mainnets
        );
        assert!(!chains.mainnets.contains(&".".to_string()));

        assert!(chains.testnets.len() > 1);
        assert!(
            chains.testnets.contains(&"cosmoshubtestnet".to_string()),
            "{:?}",
            chains.testnets
        );
        assert!(!chains.testnets.contains(&".".to_string()));
    }
}
