#![feature(proc_macro_hygiene, decl_macro)]
#![feature(format_args_capture)]
#![feature(env)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate lazy_static;

use futures::executor;
use rocket::serde::json::{json, Json, Value};
use rocket::serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;
use std::sync::RwLock;
use vaultrs::client::{VaultClient, VaultClientSettingsBuilder};
use vaultrs::kv2;

lazy_static! {
    static ref SECRET_STORE: RwLock<SecretStore> = RwLock::new(SecretStore::default());
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Secret {
    password: String,
}

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
#[serde(crate = "rocket::serde")]
pub struct SecretStore {
    url: String,
    token: String,
    path: String,
}

impl SecretStore {
    fn new(s: &SecretStore) -> SecretStore {
        SecretStore {
            url: s.url.clone(),
            token: s.token.clone(),
            path: s.path.clone(),
        }
    }
}

pub async fn get_secret_from_vault() -> Secret {
    let store = read_secret_store();
    let mut client = VaultClient::new(
        VaultClientSettingsBuilder::default()
            .address(store.url.clone())
            .token(store.token.clone())
            .build()
            .unwrap(),
    )
    .unwrap();
    kv2::read(&client, "secret", &store.path).await.unwrap()
}

#[derive(Debug, Clone)]
struct InvalidSecretStoreError {
    details: String,
}

impl InvalidSecretStoreError {
    fn new(msg: &str) -> InvalidSecretStoreError {
        InvalidSecretStoreError {
            details: msg.to_string(),
        }
    }
}

impl fmt::Display for InvalidSecretStoreError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for InvalidSecretStoreError {
    fn description(&self) -> &str {
        &self.details
    }
}

impl SecretStore {
    fn validate(&self) -> Result<(), InvalidSecretStoreError> {
        if self.path.is_empty() {
            Err(InvalidSecretStoreError::new("secret path cannot be empty"))
        } else if self.token.is_empty() {
            Err(InvalidSecretStoreError::new("token cannot be empty"))
        } else if self.url.is_empty() {
            Err(InvalidSecretStoreError::new("url cannot be empty"))
        } else {
            Ok(())
        }
    }
}

fn write_secret_store(store: SecretStore) {
    let mut s = SECRET_STORE.write().unwrap();
    *s = store;
}

fn read_secret_store() -> SecretStore {
    SecretStore::new(&(SECRET_STORE.read().unwrap()))
}

#[get("/get")]
fn get_secret_store() -> Json<SecretStore> {
    Json(read_secret_store())
}

#[post("/update", format = "json", data = "<store>")]
fn register_secret_store(store: Json<SecretStore>) -> Value {
    let valid = store.validate();
    match valid {
        Ok(v) => {
            write_secret_store(store.0);
            return json!({ "status": "updated"});
        }
        Err(e) => json!({ "status": "error",
                "reason": e.to_string(),
        }),
    }
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount(
        "/secret-store",
        routes![register_secret_store, get_secret_store],
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::rocket::local::blocking::Client;
    use rocket::http::{ContentType, Status};
    use serde_json::Result;
    use std::env;

    #[test]
    fn set_secret_store() {
        let store = SecretStore {
            url: "http://127.0.0.1:8200".to_string(),
            path: "guestowner1/workload-id/secret".to_string(),
            token: "sfjdksjfksjfkdjskfjskfjd".to_string(),
        };
        let serialized_store = serde_json::to_string(&store).unwrap();
        let rocket = rocket::build().mount("/", routes![register_secret_store, get_secret_store]);
        let client = Client::new(rocket).expect("valid rocket instance");
        let mut response = client
            .post("/update")
            .header(ContentType::JSON)
            .body(serialized_store.clone())
            .dispatch();
        assert_eq!(response.status(), Status::Ok);
        let mut response = client.get("/get").header(ContentType::JSON).dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.into_string(), serialized_store.into());
    }

    #[actix_rt::test]
    async fn get_secret() {
        let url = env::var("VAULT_ADDR").unwrap();
        let token = env::var("VAULT_TOKEN").unwrap();
        println!(
            "Executing test with VAULT_ADDR: {} VAULT_TOKEN: {}",
            url, token
        );
        write_secret_store(SecretStore {
            url: url.to_string(),
            path: "guestowner1/workload-id/secret".to_string(),
            token: token.to_string(),
        });
        let secret = get_secret_from_vault().await;
        assert_eq!(secret.password, "test".to_string());
    }
}
