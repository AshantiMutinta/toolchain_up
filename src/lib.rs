use std::process::Command;

type DefaultResult<T> = Result<T, String>;
pub fn upgrade_toolchain_if_outdated(pinned_toolchain: String) -> DefaultResult<()> {
    //get default toolchain
    let default_toolchain = get_override_toolchain()?;

    //check if toolchain matches pinned toolchain

    if default_toolchain.contains(&*pinned_toolchain) {
        Ok(())
    } else {
        //if not upgrade toolchain
        upgrade_toolchain(&*pinned_toolchain)
    }
}
fn run_rustup_command<'a>(
    args: &'a [&str],
    on_success: &'a dyn Fn() -> DefaultResult<()>,
    on_fail: &'a dyn Fn() -> DefaultResult<()>,
) -> DefaultResult<()> {
    let mut command = Command::new("rustup");
    args.iter().for_each(|s| {
        command.arg(s);
    });
    let command_result = command
        .output()
        .map_err(|_| "Could not run rustup command".to_string())?;

    if command_result.status.success() {
        on_success()
    } else {
        on_fail()?;
        Err(String::from_utf8_lossy(&command_result.stderr).to_string())
    }
}

fn upgrade_toolchain(pinned_toolchain: &str) -> DefaultResult<()> {
    //get default host
    let default_host = get_default_host()?;

    //get complete concat of toolchain + default_host
    let complete_pinned_toolchain = &*vec![pinned_toolchain, &*default_host].join("-");

    //install toolchain
    let toolchain_install_args = vec!["toolchain", "install", complete_pinned_toolchain];
    run_rustup_command(
        &toolchain_install_args,
        &|| Ok(println!("toolchain installed")),
        &|| Err("problem installing toolchain".to_string()),
    )?;

    //install wasm target
    let target_add_args = vec!["target", "add", "wasm32-unknown-unknown"];
    run_rustup_command(&target_add_args, &|| Ok(println!("target added")), &|| {
        Err("Failed to add wasm target".to_string())
    })?;

    //override directory with new wasm
    let override_rustup_args = vec!["override", "set", complete_pinned_toolchain];

    run_rustup_command(
        &override_rustup_args,
        &|| Ok(println!("target added")),
        &|| Err("Failed to add target".to_string()),
    )
}

fn get_default_host() -> DefaultResult<String> {
    let default_host_unsanitized = extract_from_rustup("Default host")?;
    let split_hosts = default_host_unsanitized.split(' ').collect::<Vec<&str>>();
    let mut split_string = split_hosts.iter();
    split_string.next();
    split_string.next();
    Ok(String::from(
        *split_string
            .next()
            .clone()
            .ok_or_else(|| "Could not get default-host")?,
    ))
}

fn get_default_toolchain() -> DefaultResult<String> {
    let default_host_unsanitized = extract_from_rustup("(default")?;
    let split_hosts = default_host_unsanitized.split(' ').collect::<Vec<&str>>();

    let mut split_string = split_hosts.iter();
    Ok(String::from(
        *split_string
            .next()
            .clone()
            .ok_or("Could not get default toolchain")?,
    ))
}

fn get_override_toolchain() -> DefaultResult<String> {
    let default_host_unsanitized = extract_from_rustup("(directory override")?;
    let split_hosts = default_host_unsanitized.split(' ').collect::<Vec<&str>>();

    if !split_hosts.is_empty() {
        let mut split_string = split_hosts.iter();
        Ok(String::from(
            *split_string
                .next()
                .clone()
                .ok_or_else(|| "incompatible meta section")?,
        ))
    } else {
        get_default_toolchain()
    }
}

fn extract_from_rustup(rustup_show_match: &str) -> DefaultResult<String> {
    let rustup_result = Command::new("rustup")
        .arg("show")
        .output()
        .map_err(|_| "Could not run rustup command".to_string())?
        .stdout;
    let std_out = String::from_utf8_lossy(&rustup_result);
    let command_split = std_out
        .split('\n')
        .filter(|s| s.contains(rustup_show_match))
        .collect::<Vec<&str>>();
    let command_result: &str = command_split
        .iter()
        .next()
        .ok_or_else(|| "Could not extract information from rustup")?;
    Ok(String::from(command_result))
}
