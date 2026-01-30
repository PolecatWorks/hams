use hams::hams::Hams;
use hams::hams::config::HamsConfig;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_http_alive_endpoint() {
    // Setup: Start HaMS on a random port (let OS decide, but hams config strictly types address)
    // For simplicity in this test environment, we'll try a likely free port, e.g., 8081.
    // Ideally, we'd bind to port 0 and get the assigned port back, but Hams structure takes config first.
    let port = 8081;
    let address = format!("127.0.0.1:{}", port).parse().unwrap();
    let config = HamsConfig {
        name: "test_service".to_string(),
        version: "0.1.0".to_string(),
        address,
        logging: true,
    };

    let mut hams = Hams::new(config);
    hams.start().expect("Failed to start HaMS");

    // Give it a moment to bind
    sleep(Duration::from_millis(100)).await;

    let client = reqwest::Client::new();
    let url = format!("http://127.0.0.1:{}/hams/alive", port);

    // Test: Valid Request
    let resp = client
        .get(&url)
        .send()
        .await
        .expect("Failed to send request");
    assert_eq!(resp.status(), reqwest::StatusCode::OK);

    let body: serde_json::Value = resp.json().await.expect("Failed to parse JSON");
    assert_eq!(body["name"], "alive");
    assert_eq!(body["valid"], true);

    // Teardown
    hams.stop().expect("Failed to stop HaMS");
}

#[tokio::test]
async fn test_http_ready_endpoint() {
    let port = 8082;
    let address = format!("127.0.0.1:{}", port).parse().unwrap();
    let config = HamsConfig {
        name: "test_service_ready".to_string(),
        version: "0.1.0".to_string(),
        address,
        logging: true,
    };

    let mut hams = Hams::new(config);
    hams.start().expect("Failed to start HaMS");
    sleep(Duration::from_millis(100)).await;

    let client = reqwest::Client::new();
    let url = format!("http://127.0.0.1:{}/hams/ready", port);

    let resp = client
        .get(&url)
        .send()
        .await
        .expect("Failed to send request");
    assert_eq!(resp.status(), reqwest::StatusCode::OK);

    let body: serde_json::Value = resp.json().await.expect("Failed to parse JSON");
    assert_eq!(body["name"], "ready");
    assert_eq!(body["valid"], true);

    hams.stop().expect("Failed to stop HaMS");
}

#[tokio::test]
async fn test_http_version_endpoint() {
    let port = 8083;
    let address = format!("127.0.0.1:{}", port).parse().unwrap();
    let config = HamsConfig {
        name: "version_test".to_string(),
        version: "1.2.3".to_string(),
        address,
        logging: true,
    };

    let mut hams = Hams::new(config);
    hams.start().expect("Failed to start HaMS");
    sleep(Duration::from_millis(100)).await;

    let client = reqwest::Client::new();
    let url = format!("http://127.0.0.1:{}/hams/version", port);

    let resp = client
        .get(&url)
        .send()
        .await
        .expect("Failed to send request");
    assert_eq!(resp.status(), reqwest::StatusCode::OK);

    let body: serde_json::Value = resp.json().await.expect("Failed to parse JSON");
    assert_eq!(body["name"], "version_test");
    assert_eq!(body["version"], "1.2.3");
    // hams_name and hams_version are dynamic from build env, just check existence
    assert!(body["hams_name"].is_string());
    assert!(body["hams_version"].is_string());

    hams.stop().expect("Failed to stop HaMS");
}

#[tokio::test]
async fn test_http_metrics_endpoint() {
    let port = 8084;
    let address = format!("127.0.0.1:{}", port).parse().unwrap();
    let config = HamsConfig {
        name: "metrics_test".to_string(),
        version: "1.0.0".to_string(),
        address,
        logging: true,
    };

    let mut hams = Hams::new(config);

    // Mock callback for metrics
    extern "C" fn mock_metrics_cb(_ptr: *const libc::c_void) -> *mut libc::c_char {
        let metrics = "mock_metric 42\n".to_string();
        std::ffi::CString::new(metrics).unwrap().into_raw()
    }

    extern "C" fn mock_metrics_free(ptr: *mut libc::c_char) {
        unsafe {
            if !ptr.is_null() {
                drop(std::ffi::CString::from_raw(ptr));
            }
        }
    }

    hams.register_prometheus(mock_metrics_cb, mock_metrics_free, std::ptr::null())
        .expect("Failed to register_prometheus");

    hams.start().expect("Failed to start HaMS");
    sleep(Duration::from_millis(100)).await;

    let client = reqwest::Client::new();
    let url = format!("http://127.0.0.1:{}/hams/metrics", port);

    let resp = client
        .get(&url)
        .send()
        .await
        .expect("Failed to request metrics");
    assert_eq!(resp.status(), reqwest::StatusCode::OK);

    let body = resp.text().await.expect("Failed to get body");
    // Verify content from our mock
    assert_eq!(body, "mock_metric 42\n");

    hams.stop().expect("Failed to stop HaMS");
}

#[tokio::test]
async fn test_concurrent_requests() {
    let port = 8085;
    let address = format!("127.0.0.1:{}", port).parse().unwrap();
    let config = HamsConfig {
        name: "concurrent_test".to_string(),
        version: "1.0.0".to_string(),
        address,
        logging: true,
    };

    let mut hams = Hams::new(config);
    hams.start().expect("Failed to start HaMS");
    sleep(Duration::from_millis(100)).await;

    let client = reqwest::Client::new();
    let url_alive = format!("http://127.0.0.1:{}/hams/alive", port);
    let url_ready = format!("http://127.0.0.1:{}/hams/ready", port);

    let mut handles = vec![];

    for _ in 0..50 {
        let client = client.clone();
        let url_alive = url_alive.clone();
        let url_ready = url_ready.clone();

        handles.push(tokio::spawn(async move {
            for _ in 0..10 {
                let resp = client.get(&url_alive).send().await.expect("Failed check");
                assert_eq!(resp.status(), reqwest::StatusCode::OK);

                let resp = client.get(&url_ready).send().await.expect("Failed check");
                assert_eq!(resp.status(), reqwest::StatusCode::OK);
            }
        }));
    }

    for handle in handles {
        handle.await.expect("Task failed");
    }

    hams.stop().expect("Failed to stop HaMS");
}
