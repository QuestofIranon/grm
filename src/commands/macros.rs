// retrieves the git_root from
macro_rules! grm_root {
    () => {{
        use git2::Config;

        let config =
            Config::open_default().expect("No git config found, do you have git installed?");

        match config.get_path("grm.root") {
            Ok(root) => root,
            Err(_error) => match config.get_path("ghq.root") {
                Ok(root) => root,
                Err(_error) => {
                    println!("grm.root not specified in git config");
                    return;
                }
            },
        }
    }};
}
