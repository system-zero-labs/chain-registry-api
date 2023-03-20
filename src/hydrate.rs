pub fn shallow_clone(remote: String, git_ref: String, path: String) -> anyhow::Result<()> {
    let mut cmd = std::process::Command::new("git");
    cmd.arg("clone")
        .arg("--depth")
        .arg("1")
        .arg("--branch")
        .arg(git_ref)
        .arg(remote)
        .arg(path);
    let output = cmd.output()?;
    if !output.status.success() {
        anyhow::bail!(
            "git clone failed with status: {:?} stderr: {:?} stdout: {:?}",
            output.status,
            std::str::from_utf8(output.stderr.as_ref()).unwrap(),
            std::str::from_utf8(output.stdout.as_ref()).unwrap(),
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::hydrate::shallow_clone;
    use tempfile::TempDir;

    #[test]
    fn test_shallow_clone() {
        let temp_dir = TempDir::new().unwrap();

        shallow_clone(
            "https://github.com/cosmos/chain-registry".to_string(),
            "master".to_string(),
            temp_dir.path().to_str().unwrap().to_string(),
        )
        .unwrap();

        assert!(temp_dir.path().join("cosmoshub/chain.json").exists());
    }
}
