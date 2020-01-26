#[macro_use]
extern crate diesel;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate gotham_derive;

use dotenv::dotenv;
use gotham::helpers::http::response::create_empty_response;
use gotham::middleware::state::StateMiddleware;
use gotham::pipeline::new_pipeline;
use gotham::pipeline::set::{finalize_pipeline_set, new_pipeline_set};
use gotham::router::builder::*;
use gotham::router::Router;
use gotham::state::State;
use gotham_middleware_diesel::{self, DieselMiddleware};
// use diesel_middleware::{DieselMiddleware, Repo};
use hyper::{Body, Response, StatusCode};

use diesel::PgConnection;
use diesel::r2d2::{ConnectionManager, PooledConnection};

mod auth;
mod handlers;
mod models;
mod schema;
// mod users;
mod conduit;

use std::env;

use auth::GoogleRedirectExtractor;

use handlers::auth::{google_authorize_handler, google_redirect_handler};

pub type Repo = gotham_middleware_diesel::Repo<PgConnection>;
pub type Connection = PooledConnection<ConnectionManager<PgConnection>>;

fn main() {
    dotenv().ok();

    let addr = "127.0.0.1:7878";
    println!("Listening for requests at http://{}", addr);
    gotham::start(addr, router())
}

fn router() -> Router {
    let database_url =
        env::var("DATABASE_URL").unwrap_or("postgresql://postgres@localhost:5432".to_string());

    // create a new repo, in this case just using a SQLite setup
    let repo = Repo::new(&database_url);

    let pipelines = new_pipeline_set();
    let (pipelines, default) = pipelines.add(
        new_pipeline()
            .add(DieselMiddleware::new(repo))
            .build(),
    );

    let pipeline_set = finalize_pipeline_set(pipelines);
    let default_chain = (default, ());

    build_router(default_chain, pipeline_set, |route| {
        route.get_or_head("/").to(index_handler);

        route.get("/google/authorize").to(google_authorize_handler);
        route
            .get("/google/redirect")
            .with_query_string_extractor::<GoogleRedirectExtractor>()
            .to(google_redirect_handler);
    })
}

fn index_handler(state: State) -> (State, Response<Body>) {
    let res = create_empty_response(&state, StatusCode::OK);

    (state, res)
}
