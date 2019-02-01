use std::process::Command;
use failure::Error;
use assert_cmd::prelude::*;
use predicates::prelude::*;

#[test]
fn invalid_address() -> Result<(), Error> {
    let mut cmd = Command::main_binary()?;
    cmd.arg("-b").arg("not_an_ip");
    cmd.assert().failure().stderr(predicate::str::contains("expected a valid hostname"));
    Ok(())
}

#[test]
fn invalid_port() -> Result<(), Error> {
    let mut cmd = Command::main_binary()?;
    let big_port_arg = format!("{}", 2u32.pow(16));
    cmd.arg("-p").arg(big_port_arg);
    cmd.assert().failure().stderr(predicate::str::contains("port"));
    Ok(())
}
