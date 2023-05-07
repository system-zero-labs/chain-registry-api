use std::fs;
use std::path::PathBuf;

pub struct ChainRegRepo {
    pub commit: String,
    pub mainnets: Vec<PathBuf>,
    pub testnets: Vec<PathBuf>,
}

pub fn shallow_clone(
    remote: String,
    git_ref: String,
    clone_dir: &PathBuf,
) -> anyhow::Result<ChainRegRepo> {
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

    // Get commit hash
    let mut cmd = std::process::Command::new("git");
    let output = cmd
        .current_dir(clone_dir)
        .arg("rev-parse")
        .arg("HEAD")
        .output()?;
    if !output.status.success() {
        anyhow::bail!(
            "git rev-parse failed with status: {:?} stderr: {:?} stdout: {:?}",
            output.status,
            std::str::from_utf8(output.stderr.as_ref()).unwrap_or("cannot read stderr"),
            std::str::from_utf8(output.stdout.as_ref()).unwrap_or("cannot read stdout")
        );
    }
    let commit = std::str::from_utf8(output.stdout.as_ref())?;
    let commit = commit.trim().to_string();

    Ok(ChainRegRepo {
        commit,
        mainnets,
        testnets,
    })
}

fn collect_chains(dir: PathBuf) -> anyhow::Result<Vec<PathBuf>> {
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
            !(fname.starts_with("_") || fname.starts_with(".") || fname.starts_with("testnets"))
        })
        .collect();
    Ok(found)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_shallow_clone() {
        let temp_dir = TempDir::new().unwrap();

        let repo = shallow_clone(
            "https://github.com/cosmos/chain-registry".to_string(),
            "master".to_string(),
            &temp_dir.path().to_path_buf(),
        )
        .unwrap();

        assert!(!repo.commit.is_empty());
        assert!(repo.commit.chars().all(|c| c.is_ascii_hexdigit()));

        assert!(temp_dir.path().join("cosmoshub/chain.json").exists());

        assert!(repo.mainnets.len() > 1);
        assert!(repo.mainnets.iter().all(|p| p.exists() && p.is_dir()));

        let mainnets: Vec<String> = repo
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
        assert!(!mainnets.contains(&".git".to_string()), "{:?}", mainnets);
        assert!(!mainnets.contains(&".github".to_string()), "{:?}", mainnets);
        assert!(
            !mainnets.contains(&"testnets".to_string()),
            "{:?}",
            mainnets
        );

        assert!(repo.testnets.len() > 1);
        assert!(repo.testnets.iter().all(|p| p.exists() && p.is_dir()));

        let testnets: Vec<String> = repo
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
