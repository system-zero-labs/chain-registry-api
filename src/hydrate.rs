use std::fs;
use std::path;

pub struct ChainDirs {
    mainnets: Vec<path::PathBuf>,
    testnets: Vec<path::PathBuf>,
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

    let mainnets = collect_chains(clone_dir.clone())?;
    let testnets = collect_chains(clone_dir.join("testnets"))?;
    Ok(ChainDirs { mainnets, testnets })
}

fn collect_chains(dir: path::PathBuf) -> anyhow::Result<Vec<path::PathBuf>> {
    let found = fs::read_dir(dir)?
        .filter_map(|f| {
            let f = f.unwrap().path();
            if !f.is_dir() {
                return None;
            }
            return Some(f);
        })
        .filter(|f| {
            let fname = f.file_name().unwrap().to_str().unwrap();
            !(fname.starts_with("_") || fname.starts_with("testnets"))
        })
        .collect();
    Ok(found)
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
        assert!(chains.mainnets.iter().all(|p| p.exists() && p.is_dir()));

        let mainnets: Vec<String> = chains
            .mainnets
            .iter()
            .map(|p| p.file_name().unwrap().to_str().unwrap().to_string())
            .collect();

        assert!(
            mainnets.contains(&"cosmoshub".to_string()),
            "{:?}",
            mainnets
        );
        assert!(mainnets.contains(&"osmosis".to_string()), "{:?}", mainnets);
        assert!(!mainnets.contains(&".".to_string()), "{:?}", mainnets);
        assert!(
            !mainnets.contains(&"testnets".to_string()),
            "{:?}",
            mainnets
        );

        assert!(chains.testnets.len() > 1);
        assert!(chains.testnets.iter().all(|p| p.exists() && p.is_dir()));

        let testnets: Vec<String> = chains
            .testnets
            .iter()
            .map(|p| p.file_name().unwrap().to_str().unwrap().to_string())
            .collect();

        assert!(testnets.len() > 1);
        assert!(
            testnets.contains(&"cosmoshubtestnet".to_string()),
            "{:?}",
            testnets
        );
        assert!(!testnets.contains(&".".to_string()));
    }
}
