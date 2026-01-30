# HaMS API Reference

HaMS (Health and Monitoring System) exposes a lightweight HTTP server to provide health checks, readiness probes, version information, and metrics.

**Default Port**: `8080` (Configurable via `HamsConfig`)

## Endpoints

### Health & Readiness

#### `GET /hams/alive`
**Description**: Liveness probe. Checks if the service is running and all "alive" checks are passing.
**Responses**:
- `200 OK`: Service is healthy.
- `503 Service Unavailable`: Service is unhealthy (one or more probes failed).

**Response Body (JSON)**:
```json
{
  "name": "HamName",
  "valid": true
}
```

#### `GET /hams/ready`
**Description**: Readiness probe. Checks if the service is ready to accept traffic and all "ready" checks are passing.
**Responses**:
- `200 OK`: Service is ready.
- `503 Service Unavailable`: Service is not ready.

**Response Body (JSON)**:
```json
{
  "name": "HamName",
  "valid": true
}
```

#### `GET /hams/alive_verbose` / `GET /hams/ready_verbose`
**Description**: Detailed version of the probes, listing the status of each individual check.
**Responses**: `200` or `503`.

**Response Body (JSON)**:
```json
{
  "name": "HamName",
  "valid": true,
  "details": [
    { "name": "database_connection", "valid": true },
    { "name": "cache_warmup", "valid": false }
  ]
}
```

### Lifecycle

#### `GET /hams/version`
**Description**: Returns version information for the service and the HaMS library itself.
**Responses**:
- `200 OK`

**Response Body (JSON)**:
```json
{
  "name": "Service Name",
  "version": "1.0.0",
  "hams_name": "HaMS",
  "hams_version": "0.1.0"
}
```

#### `POST /hams/shutdown`
**Description**: Triggers the registered shutdown callback in the host application. Used for graceful shutdown requests.
**Responses**:
- `200 OK`: Shutdown signal received.
- `500 Internal Server Error`: Failed to trigger shutdown.

### Metrics

#### `GET /hams/metrics`
**Description**: Exposes metrics in Prometheus text format (or any format supported by the registered callback).
**Responses**:
- `200 OK`: Metrics retrieved successfully.
- `500 Internal Server Error`: Accessing metrics failed (e.g., locking issue, callback implementation error).

**Response Format**: `text/plain`

## Error Handling

The API returns appropriate HTTP status codes for various failure conditions:

| Status Code | Reason | Description |
| :--- | :--- | :--- |
| `409 Conflict` | Already Running | The service or a component is already active. |
| `503 Service Unavailable` | Cancelled | The operation was cancelled. |
| `406 Not Acceptable` | Probe Not Good | The requested probe name is invalid. |
| `500 Internal Server Error` | Callback Error | The external callback (FFI) failed. |
| `500 Internal Server Error` | Join Error | Failed to join an internal thread. |
| `500 Internal Server Error` | FFI Error | A low-level FFI error occurred (e.g., null pointer, buffer size). |
