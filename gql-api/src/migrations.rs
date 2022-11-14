use diesel::pg::PgConnection;
use diesel::prelude::*;

use crate::config::PostgresConfig;

diesel_migrations::embed_migrations!("./diesel/migrations");

pub fn run(db_config: &PostgresConfig) {
    let database_url = db_config.connection_string();
    let connection = PgConnection::establish(&database_url).expect("Connection failed");
    embedded_migrations::run(&connection).expect("Migrations failed");
}
