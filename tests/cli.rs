use assert_cmd::Command;
use failure::Error;
use predicates::prelude::*;

#[test]
fn invalid_address() -> Result<(), Error> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    let assert = cmd.arg("-b").arg("not_an_ip").assert();
    assert
        .failure()
        .stderr(predicate::str::contains("expected a valid hostname"));
    Ok(())
}

#[test]
fn invalid_port() -> Result<(), Error> {
    let big_port_arg = format!("{}", 2u32.pow(16));
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    let assert = cmd.arg("-p").arg(big_port_arg).assert();
    assert.failure().stderr(predicate::str::contains("port"));
    Ok(())
}
