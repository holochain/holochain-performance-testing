use hdk::prelude::*;

#[hdk_extern]
fn init() -> ExternResult<InitCallbackResult> {
    create_cap_grant(CapGrantEntry {
        tag: "access".into(),
        access: CapAccess::Unrestricted,
        functions: GrantedFunctions::Listed(BTreeSet::from([(
            "remote_ping_coordinator".into(),
            "ping".into(),
        )])),
    })?;

    Ok(InitCallbackResult::Pass)
}

#[hdk_extern]
fn ping(timestamp: i64) -> ExternResult<i64> {
    Ok(timestamp)
}
