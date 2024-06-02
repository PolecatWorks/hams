use std::convert::Infallible;

use warp::{
    hyper::StatusCode,
    reject::{Reject, Rejection},
    reply::{json, Reply},
    Filter,
};

use crate::{error::HamsError, hams::Hams, health::check::HealthCheck};

impl Reject for HamsError {}

async fn handle_rejection(err: Rejection) -> std::result::Result<impl Reply, Infallible> {
    let (code, json_message) = if err.is_not_found() {
        (StatusCode::NOT_FOUND, json(&"Not Found".to_string()))
    } else if let Some(e) = err.find::<HamsError>() {
        match e {
            HamsError::Message(msg) => (StatusCode::BAD_REQUEST, json(&msg)),
            HamsError::PoisonError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                json(&"Poison Error".to_string()),
            ),
            HamsError::Unknown => (
                StatusCode::INTERNAL_SERVER_ERROR,
                json(&"Unknown Error".to_string()),
            ),
            HamsError::NotRunning => (
                StatusCode::INTERNAL_SERVER_ERROR,
                json(&"Not Running".to_string()),
            ),
            HamsError::SendError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                json(&"Send Error".to_string()),
            ),
            HamsError::IoError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                json(&"IO Error".to_string()),
            ),
            HamsError::AlreadyRunning => todo!(),
            HamsError::Cancelled => todo!(),
            HamsError::CallbackError => todo!(),
            HamsError::JoinError2 => todo!(),
            HamsError::JoinError(_) => todo!(),
            HamsError::NoThread => todo!(),
            HamsError::NulError(_) => todo!(),
            HamsError::ProbeNotGood(probename) => (StatusCode::NOT_ACCEPTABLE, json(probename)),
            HamsError::PreflightCheck => todo!(),
            HamsError::ShutdownCheck => todo!(),
            // Add match arms for the remaining error variants here
        }
    } else {
        eprintln!("unhandled error: {:?}", err);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            json(&"Internal Server Error".to_string()),
        )
    };

    Ok(warp::reply::with_status(json_message, code))
}

pub fn hams_service(
    hams: Hams,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let shutdown = warp::path("shutdown")
        .and(with_hams(hams.clone()))
        .and_then(handlers::shutdown_handler);

    let alive = warp::path("alive")
        .and(with_healthcheck(hams.alive.clone()))
        .and_then(handlers::check_handler);

    let alive_verbose = warp::path("alive_verbose")
        .and(with_healthcheck(hams.alive.clone()))
        .and_then(handlers::check_verbose_handler);

    let ready = warp::path("ready")
        .and(with_healthcheck(hams.ready.clone()))
        .and_then(handlers::check_handler);

    let ready_verbose = warp::path("ready_verbose")
        .and(with_healthcheck(hams.ready.clone()))
        .and_then(handlers::check_verbose_handler);

    let version = warp::path("version")
        .and(warp::get())
        .and(with_hams(hams.clone()))
        .and_then(handlers::version);

    let metrics = warp::path("metrics")
        .and(warp::get())
        .and(with_hams(hams.clone()))
        .and_then(handlers::metrics);

    warp::path("hams").and(
        version
            .or(shutdown)
            .or(alive)
            .or(ready)
            .or(alive_verbose)
            .or(ready_verbose)
            .or(metrics)
            .recover(handle_rejection),
    )
}

fn with_healthcheck(
    check: HealthCheck,
) -> impl Filter<Extract = (HealthCheck,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || check.clone())
}

fn with_hams(
    hams: Hams,
) -> impl Filter<Extract = (Hams,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || hams.clone())
}

mod handlers {
    use std::ffi::CStr;

    use log::{error, info};

    use crate::{error::HamsError, health::check::HealthCheck};

    use super::Hams;
    use serde::Serialize;
    use tokio::time::Instant;
    use warp::{http::Response, reject::Rejection};

    /// Reply structure for Version response
    #[derive(Serialize)]
    struct VersionReply {
        name: String,
        version: String,
        hams_name: String,
        hams_version: String,
    }

    /// Handler for version endpoint
    pub async fn version(hams: Hams) -> Result<impl warp::Reply, Rejection> {
        let version_reply = VersionReply {
            name: hams.name,
            version: hams.version,
            hams_version: hams.hams_version,
            hams_name: hams.hams_name,
        };
        Ok(warp::reply::json(&version_reply))
    }

    /// Handler for shutdown endpoint
    pub async fn shutdown_handler(hams: Hams) -> Result<impl warp::Reply, Rejection> {
        // TODO: Call shutdown
        // Hams::tigger_callback(hams.shutdown_cb.clone());

        version(hams).await
    }

    /// Handler for alive endpoint
    pub async fn check_handler(check: HealthCheck) -> Result<impl warp::Reply, Rejection> {
        let health_check = check.check(Instant::now()).await;

        let valid = health_check.valid;
        Ok(warp::reply::with_status(
            health_check,
            if valid {
                warp::http::StatusCode::OK
            } else {
                warp::http::StatusCode::SERVICE_UNAVAILABLE
            },
        ))
    }

    /// Handler for alive endpoint
    pub async fn check_verbose_handler(check: HealthCheck) -> Result<impl warp::Reply, Rejection> {
        let health_check = check.check_verbose(Instant::now()).await;

        let valid = health_check.valid;

        Ok(warp::reply::with_status(
            health_check,
            if valid {
                warp::http::StatusCode::OK
            } else {
                warp::http::StatusCode::SERVICE_UNAVAILABLE
            },
        ))
    }

    /// Handler for metrics endpoint
    pub async fn metrics(hams: Hams) -> Result<impl warp::Reply, Rejection> {
        let x = hams
            .prometheus_cb
            .lock()
            .map_err(|_e| HamsError::PoisonError)?;

        let metrics = match *x {
            Some(ref cb) => {
                info!("Metrics are here");

                let c_string = (cb.my_cb)(cb.state);

                let c_string_2 = unsafe { CStr::from_ptr(c_string) };
                let metric_response = c_string_2.to_str().unwrap().to_string();

                (cb.my_cb_free)(c_string);

                metric_response
                // "Metrics go here".to_string()
            }
            None => {
                info!("Metrics are NOT here");
                "No metrics registered".to_string()
            }
        };

        Response::builder()
            .header("Content-Type", "text/plain; version=0.0.4")
            .body(metrics)
            .map_err(|e| {
                error!("Error building response: {}", e);
                warp::reject::reject()
            })
    }

    #[cfg(test)]
    mod tests {
        use crate::webservice::hams_service;

        use super::*;
        use warp::http::StatusCode;

        /// Test metrics
        /// This test will fail as the metrics handler works with empty reply when nothing is registered
        #[tokio::test]
        #[cfg_attr(miri, ignore)]
        async fn test_metrics() {
            let hams = Hams::new("test");
            let api = hams_service(hams);

            let reply = warp::test::request()
                .method("GET")
                .path("/hams/metrics")
                .reply(&api)
                .await;

            assert_eq!(reply.status(), StatusCode::OK);
        }

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
