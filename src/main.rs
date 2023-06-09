extern crate casper_client;
extern crate dotenv;

use casper_client::PaymentStrParams;
use casper_client::{DeployStrParams, SessionStrParams};
use dotenv::dotenv;
use rocket::serde::json::Json;
use rocket::serde::{Deserialize, Serialize};
use rocket::State;
use rocket::http::Header;
use rocket::{Request, Response};
use rocket::fairing::{Fairing, Info, Kind};
use std::collections::HashMap;
use std::env;
use std::path::Path;

#[macro_use]
extern crate rocket;

const CONTRACT_PACKAGE_HASH: &str =
  "691c5d3789896a66458bbf0decf3ac96e067af0453ccffe7fe295e5b244010b9";
const CONTRACT_OWNER_PUBLIC_KEY: &str =
  "01e69ae401815ca564277d4d30ba1fb53c6b866e20a153ea9d003d75f3b96a6f62";

pub struct CORS;

#[rocket::async_trait]
impl Fairing for CORS {
  fn info(&self) -> Info {
    Info {
      name: "Add CORS headers to responses",
      kind: Kind::Response
    }
  }

  async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
    response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
    response.set_header(Header::new("Access-Control-Allow-Methods", "POST, GET, PATCH, OPTIONS"));
    response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
    response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
  }
}

struct AppConfig {
  node_rpc: String,
  chain: String,
  secret_key_path: String,
}

#[get("/")]
fn index() -> &'static str {
  "Nothing to see here."
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct User {
  account_hash: String,
  metadata: String,
  fingerprint: String,
}

#[post("/claim-nft", format = "json", data = "<user>")]
async fn claim_nft(user: Json<User>, state: &State<AppConfig>) -> &'static str {
  // Validate user data.
  if user.metadata.len() > 64 {
    return "Metadata cannot be longer than 64 characters.";
  }
  let metadata = user.metadata.replace('"', "'");

  // Register owner.
  let token_owner_arg = format!("token_owner:key='account-hash-{}'", &user.account_hash);
  let session_args: Vec<&str> = vec![&token_owner_arg];
  let deploy_params = DeployStrParams {
    secret_key: &state.secret_key_path,
    timestamp: "",
    ttl: "50s",
    gas_price: "1",
    chain_name: &state.chain,
    dependencies: Vec::new(),
    session_account: CONTRACT_OWNER_PUBLIC_KEY,
  };
  let session_params = SessionStrParams::with_package_hash(
    CONTRACT_PACKAGE_HASH,
    "",
    "register_owner",
    session_args,
    "",
  );
  let payment_params = PaymentStrParams::with_amount("2000000000");
  let deploy_result = casper_client::put_deploy(
    "",
    &state.node_rpc,
    0,
    deploy_params,
    session_params,
    payment_params,
  )
  .await;

  // Handle result.
  if let Err(e) = deploy_result {
    println!("Error while registering owner: {}", e);
    return "Unable to register owner.";
  }

  // Mint token.
  let token_owner_arg = format!("token_owner:key='account-hash-{}'", &user.account_hash);
  let token_meta_data_arg = format!(
    "token_meta_data:string='{{\"user_message\": \"{}\"}}'",
    &metadata
  );
  let session_args: Vec<&str> = vec![&token_owner_arg, &token_meta_data_arg];
  let deploy_params = DeployStrParams {
    secret_key: &state.secret_key_path,
    timestamp: "",
    ttl: "50s",
    gas_price: "1",
    chain_name: &state.chain,
    dependencies: Vec::new(),
    session_account: CONTRACT_OWNER_PUBLIC_KEY,
  };
  let session_params =
    SessionStrParams::with_package_hash(CONTRACT_PACKAGE_HASH, "", "mint", session_args, "");
  let payment_params = PaymentStrParams::with_amount("22000000000");
  let deploy_result = casper_client::put_deploy(
    "",
    &state.node_rpc,
    0,
    deploy_params,
    session_params,
    payment_params,
  )
  .await;

  // Handle result.
  match deploy_result {
    Err(e) => {
      println!("Error while minting token: {}", e);
      return "Unable to register owner.";
    },
    Ok(result) => {
      println!("[Fingerprint: {}] Mint call result: {:?}", user.fingerprint, result.get_result());
    }
  }

  "OK"
}

#[launch]
async fn rocket() -> _ {
  // Parse args.
  dotenv().ok();
  let env_vars = env::vars().collect::<HashMap<String, String>>();
  let node_rpc = env_vars
    .get("CASPER_NODE_RPC")
    .expect("CASPER_NODE_RPC must be configured.");
  let chain = env_vars
    .get("CASPER_NET")
    .expect("CASPER_NET must be configured.");
  let secret_key_path = env_vars
    .get("SECRET_KEY_PATH")
    .expect("SECRET_KEY_PATH must be configured.");

  // Process args.
  if !Path::new(secret_key_path).is_file() {
    panic!("Secret key file does not exist.");
  }
  if chain != "casper" && chain != "casper-test" {
    panic!("Invalid chain specified.");
  }
  casper_client::get_state_root_hash("", node_rpc, 0, "")
    .await
    .expect("Node's RPC is not responding.");

  // Launch REST app.
  let app_config = AppConfig {
    node_rpc: node_rpc.clone(),
    chain: chain.clone(),
    secret_key_path: secret_key_path.clone(),
  };
  rocket::build()
    .mount("/", routes![index, claim_nft])
    .manage(app_config)
    .attach(CORS)
}
