const HAPP: &[u8] = include_bytes!("../../../zomes/remote_ping_zome/remote_ping_zome.happ");

use trycp_client::Request;

const ONE_MIN: std::time::Duration = std::time::Duration::from_secs(60);
const ALICE: &str = "alice";

use std::collections::HashMap;
use std::io::Result;

trait CliExt {
    async fn admin(&self, id: String, r: holochain_conductor_api::AdminRequest) -> Result<holochain_conductor_api::AdminResponse>;
}

impl CliExt for trycp_client::TrycpClient {
    async fn admin(&self, id: String, r: holochain_conductor_api::AdminRequest) -> Result<holochain_conductor_api::AdminResponse> {
        let i = rmp_serde::to_vec_named(&r).map_err(std::io::Error::other)?;
        let i = self.request(Request::CallAdminInterface {
            id,
            message: i,
        }, ONE_MIN).await?.into_bytes();
        let i: holochain_conductor_api::AdminResponse = rmp_serde::from_slice(&i).map_err(std::io::Error::other)?;
        Ok(i)
    }
}

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let (c, _r) = trycp_client::TrycpClient::connect("ws://127.0.0.1:9000").await.unwrap();

    c.request(Request::Reset, ONE_MIN).await.unwrap();
    c.request(Request::ConfigurePlayer {
        id: ALICE.to_string(),
        partial_config: "".to_string(),
    }, ONE_MIN).await.unwrap();
    c.request(Request::Startup {
        id: ALICE.to_string(),
        log_level: None,
    }, ONE_MIN).await.unwrap();

    let i = c.admin(ALICE.to_string(), holochain_conductor_api::AdminRequest::GenerateAgentPubKey).await.unwrap();
    let agent_key = match i {
        holochain_conductor_api::AdminResponse::AgentPubKeyGenerated(k) => k,
        _ => panic!(),
    };
    println!("{agent_key:?}");

    let i = holochain_types::app::AppBundle::decode(HAPP).unwrap();
    let i = holochain_types::app::AppBundleSource::Bundle(i);
    let i = holochain_types::app::InstallAppPayload {
        source: i,
        agent_key,
        installed_app_id: Some("remote_ping_zome".to_string()),
        membrane_proofs: HashMap::new(),
        network_seed: None,
    };
    let i = holochain_conductor_api::AdminRequest::InstallApp(Box::new(i));
    let i = c.admin(ALICE.to_string(), i).await.unwrap();
    println!("{i:#?}");

    let i = c.admin(ALICE.to_string(), holochain_conductor_api::AdminRequest::EnableApp {
        installed_app_id: "remote_ping_zome".to_string(),
    }).await.unwrap();
    println!("{i:#?}");
    let cell_id = match i {
        holochain_conductor_api::AdminResponse::AppEnabled { app: holochain_conductor_api::AppInfo { cell_info, .. }, .. } => {
            let info = cell_info.iter().next().unwrap().1.get(0).unwrap();
            match info {
                holochain_conductor_api::CellInfo::Provisioned(holochain_conductor_api::ProvisionedCell { cell_id, .. }) => cell_id.clone(),
                _ => panic!(),
            }
        }
        _ => panic!(),
    };
    println!("{cell_id:#?}");

    let i = c.admin(ALICE.to_string(), holochain_conductor_api::AdminRequest::IssueAppAuthenticationToken(holochain_conductor_api::IssueAppAuthenticationTokenPayload {
        installed_app_id: "remote_ping_zome".to_string(),
        expiry_seconds: 0,
        single_use: false,
    })).await.unwrap();
    let token = match i {
        holochain_conductor_api::AdminResponse::AppAuthenticationTokenIssued(holochain_conductor_api::AppAuthenticationTokenIssued { token, .. }) => token,
        _ => panic!(),
    };
    println!("{token:#?}");

    let i = c.admin(ALICE.to_string(), holochain_conductor_api::AdminRequest::AttachAppInterface {
        port: None,
        allowed_origins: holochain_types::websocket::AllowedOrigins::Any,
        installed_app_id: Some("remote_ping_zome".to_string()),
    }).await.unwrap();
    let app_port = match i {
        holochain_conductor_api::AdminResponse::AppInterfaceAttached { port } => port,
        _ => panic!(),
    };
    println!("app_port: {app_port}");

    c.request(Request::ConnectAppInterface {
        token: token.clone(),
        port: app_port,
    }, ONE_MIN).await.unwrap();

    let sign = ed25519_dalek::SigningKey::generate(&mut rand::thread_rng());
    let pk = holochain_types::prelude::AgentPubKey::from_raw_32(sign.verifying_key().as_bytes().to_vec());

    let mut nonce = [0; 32];
    use rand::Rng;
    rand::thread_rng().fill(&mut nonce[..]);

    let i = holochain_types::prelude::ZomeCallUnsigned {
        provenance: pk.clone(),
        cell_id,
        zome_name: "remote_ping_coordinator".to_string().into(),
        fn_name: holochain_types::prelude::FunctionName("ping".to_string()),
        cap_secret: None,
        payload: holochain_types::prelude::ExternIO::encode(42_i64).unwrap(),
        nonce: nonce.into(),
        expires_at: (holochain_types::prelude::Timestamp::now() + std::time::Duration::from_secs(4 + 60)).unwrap(),
    };
    let to_sign = i.data_to_sign().unwrap();
    use ed25519_dalek::Signer;
    let signature = sign.sign(&to_sign).to_bytes();
    let i = holochain_conductor_api::ZomeCall {
        cell_id: i.cell_id,
        zome_name: i.zome_name,
        fn_name: i.fn_name,
        payload: i.payload,
        cap_secret: i.cap_secret,
        provenance: i.provenance,
        signature: holochain_types::prelude::Signature(signature),
        nonce: i.nonce,
        expires_at: i.expires_at,
    };
    let i = holochain_conductor_api::AppRequest::CallZome(Box::new(i));
    let i = rmp_serde::to_vec_named(&i).map_err(std::io::Error::other).unwrap();
    let i = Request::CallAppInterface {
        port: app_port,
        message: i,
    };
    let i = c.request(i, ONE_MIN).await.unwrap().into_bytes();
    let i: holochain_conductor_api::AppResponse = rmp_serde::from_slice(&i).unwrap();
    println!("{i:#?}");

    c.request(Request::Reset, ONE_MIN).await.unwrap();
}

