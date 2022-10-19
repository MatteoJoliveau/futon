use std::collections::HashMap;

use testcontainers::{core::WaitFor, Image};

const NAME: &str = "public.ecr.aws/docker/library/couchdb";
const TAG: &str = "3";

pub const USERNAME: &str = "futon";
pub const PASSWORD: &str = "futon";

#[derive(Debug, Clone)]
pub struct CouchDb {
    env_vars: HashMap<String, String>,
}

impl Default for CouchDb {
    fn default() -> Self {
        let mut env_vars = HashMap::new();
        env_vars.insert("COUCHDB_USER".to_string(), USERNAME.to_string());
        env_vars.insert("COUCHDB_PASSWORD".to_string(), PASSWORD.to_string());

        Self { env_vars }
    }
}

impl Image for CouchDb {
    type Args = ();

    fn name(&self) -> String {
        NAME.to_string()
    }

    fn tag(&self) -> String {
        TAG.to_string()
    }

    fn ready_conditions(&self) -> Vec<WaitFor> {
        vec![WaitFor::message_on_stderr(
            "Apache CouchDB has started. Time to relax.",
        )]
    }

    fn env_vars(&self) -> Box<dyn Iterator<Item = (&String, &String)> + '_> {
        Box::new(self.env_vars.iter())
    }
}
