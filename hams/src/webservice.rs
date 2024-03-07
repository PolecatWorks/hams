use warp::Filter;

use crate::hams::Hams;

pub fn hams_service(
    hams: Hams,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let shutdown = warp::path("shutdown")
        .and(with_hams(hams.clone()))
        .and_then(handlers::shutdown_handler);

    let alive = warp::path("alive")
        .and(with_hams(hams.clone()))
        .and_then(handlers::alive_handler);

    let ready = warp::path("ready")
        .and(with_hams(hams.clone()))
        .and_then(handlers::ready_handler);

    let version = warp::path("version")
        .and(warp::get())
        .and(with_hams(hams.clone()))
        .and_then(handlers::version);

    warp::path("hams").and(version.or(shutdown).or(alive).or(ready))
}

fn with_hams(
    hams: Hams,
) -> impl Filter<Extract = (Hams,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || hams.clone())
}

mod handlers {
    use super::Hams;
    use serde::Serialize;
    use std::convert::Infallible;
    use warp::reply::{Reply, Response};

    /// Reply structure for Version response
    #[derive(Serialize)]
    struct VersionReply {
        name: String,
        version: String,
        hams_name: String,
        hams_version: String,
    }

    /// Handler for version endpoint
    pub async fn version(hams: Hams) -> Result<impl warp::Reply, Infallible> {
        let version_reply = VersionReply {
            name: hams.name,
            version: hams.version,
            hams_version: hams.hams_version,
            hams_name: hams.hams_name,
        };
        Ok(warp::reply::json(&version_reply))
    }

    /// Handler for shutdown endpoint
    pub async fn shutdown_handler(hams: Hams) -> Result<impl warp::Reply, Infallible> {
        Hams::tigger_callback(hams.shutdown_cb.clone());

        version(hams).await
    }

    /// Handler for alive endpoint
    pub async fn alive_handler(hams: Hams) -> Result<impl warp::Reply, Infallible> {
        // let (valid, content) = hams.check_alive();
        let (valid, content) = (true, "Alive");

        Ok(warp::reply::with_status(
            content,
            if valid {
                warp::http::StatusCode::OK
            } else {
                warp::http::StatusCode::NOT_ACCEPTABLE
            },
        ))
    }

    /// Handler for ready endpoint
    pub async fn ready_handler(hams: Hams) -> Result<impl warp::Reply, Infallible> {
        // let (valid, content) = hams.check_ready();
        let (valid, content) = (true, "Ready");

        Ok(warp::reply::with_status(
            content,
            if valid {
                warp::http::StatusCode::OK
            } else {
                warp::http::StatusCode::NOT_ACCEPTABLE
            },
        ))
    }

    #[cfg(test)]
    mod tests {
        use crate::webservice::hams_service;

        use super::*;
        use warp::{filters::reply, http::StatusCode};

        #[tokio::test]
        #[cfg_attr(miri, ignore)]
        async fn test_version() {
            let hams = Hams::new("test");
            let api = hams_service(hams);

            let reply = warp::test::request()
                .method("GET")
                .path("/hams/version")
                .reply(&api)
                .await;

            assert_eq!(reply.status(), StatusCode::OK);
        }

        #[tokio::test]
        #[cfg_attr(miri, ignore)]
        async fn test_shutdown() {
            let hams = Hams::new("test");
            let api = hams_service(hams);

            let reply = warp::test::request()
                .method("POST")
                .path("/hams/shutdown")
                .reply(&api)
                .await;

            assert_eq!(reply.status(), StatusCode::OK);
        }

        #[tokio::test]
        #[cfg_attr(miri, ignore)]
        async fn test_alive() {
            let hams = Hams::new("test");
            let api = hams_service(hams);

            let reply = warp::test::request()
                .method("GET")
                .path("/hams/alive")
                .reply(&api)
                .await;

            assert_eq!(reply.status(), StatusCode::OK);
        }

        #[tokio::test]
        #[cfg_attr(miri, ignore)]
        async fn test_ready() {
            let hams = Hams::new("test");
            let api = hams_service(hams);

            let reply = warp::test::request()
                .method("GET")
                .path("/hams/ready")
                .reply(&api)
                .await;

            assert_eq!(reply.status(), StatusCode::OK);
        }
    }
}
