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
        panic!("TODO");
        // return Err("test");
    }
    Ok(())
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_shallow_clone() {
        assert_eq!(true, false);
    }
}
