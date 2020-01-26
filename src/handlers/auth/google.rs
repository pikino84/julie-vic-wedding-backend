use gotham::helpers::http::response::{
    create_response, create_empty_response, create_temporary_redirect,
};
use gotham::handler::{HandlerError, HandlerFuture, IntoHandlerError};
use gotham::state::{FromState, State};
use hyper::{Body, Response, StatusCode};
use futures::{future, Future, Stream};

use crate::auth::{
    build_google_client, exchange_token, gen_authorize_url, get_user_profile,
    GoogleRedirectExtractor,
};
use crate::conduit::users::find_or_create_byProfile_repo;
use crate::Repo;

pub fn google_authorize_handler(state: State) -> (State, Response<Body>) {
    // TODO: Move to state.
    let google_client = build_google_client();
    let (authorize_url, _) = gen_authorize_url(google_client);

    let res = create_temporary_redirect(&state, authorize_url.to_string());

    (state, res)
}

pub fn google_redirect_handler(mut state: State) -> Box<HandlerFuture> {
    let query_param = GoogleRedirectExtractor::take_from(&mut state);
    let google_client = build_google_client();
    let token = exchange_token(&query_param, &google_client);
    let profile = get_user_profile(&token).expect("Couldn't get user's profile");

    let repo = Repo::borrow_from(&state).clone();
    let results = find_or_create_byProfile_repo(repo, profile).then(|result| match result {
        Ok(user) => {
            let res = create_response(
                &state,
                StatusCode::OK,
                mime::APPLICATION_JSON,
                serde_json::to_vec(&user).expect("Couldn't serialize query param"),
            );

            future::ok((state, res))
        },
        Err(_e) => {
            let res = create_empty_response(&state, StatusCode::INTERNAL_SERVER_ERROR);
            future::ok((state, res))
        }
    });

    Box::new(results)
}
