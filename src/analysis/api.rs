pub fn get_router() -> axum::Router {
    axum::Router::new()
        .nest("/api/v1", get_api_router())
}

pub fn get_api_router() -> axum::Router {
    axum::Router::new()
        .route("/stuckthread", axum::routing::get(stuckthread))
}
