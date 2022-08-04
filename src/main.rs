#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate lazy_static;

use rocket::serde::json::{json, Json, Value};
use rocket::serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;
use std::sync::RwLock;

lazy_static! {
    static ref SECRET_STORE: RwLock<SecretStore> = RwLock::new(SecretStore::default());
}

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(crate = "rocket::serde")]
pub struct SecretStore {
    url: String,
    token: String,
    path: String,
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

#[post("/update", format = "json", data = "<store>")]
fn register_secret_store(store: Json<SecretStore>) -> Value {
    let valid = store.validate();
    match valid {
        Ok(v) => {
            let mut s = SECRET_STORE.write().unwrap();
            *s = store.0;
            return json!({ "status": "updated"});
        }
        Err(e) => json!({ "status": "error",
                "reason": e.to_string(),
        }),
    }
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/secret-store", routes![register_secret_store])
}
