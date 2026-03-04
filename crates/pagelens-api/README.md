# pagelens-api routes

Base URL (when running `pagelens --serve`): `http://127.0.0.1:8787`

## Health

- `GET /health`
- Returns service status.
- Response: `{ "status": "ok" }`

## Start analysis

- `POST /api/runs/analyse`
- Starts a new analysis run asynchronously and returns a run id.
- Request body:
  - `run_id?: string`
  - `url: string`
  - `options?: { include_html, include_accessibility_tree, include_performance_timing, include_computed_styles, include_network_metadata }`
- Response: `{ "run_id": "..." }`

## Get run

- `GET /api/runs/{run_id}`
- Returns run metadata, status, progress, and summary.
- Response: `AnalysisRun` JSON object.

## Edit run

- `PATCH /api/runs/{run_id}`
- Updates editable run metadata.
- Request body:
  - `name?: string`
- Response: updated `AnalysisRun` JSON object.

## Get run pages

- `GET /api/runs/{run_id}/pages`
- Returns per-page analysis results for the run.
- Response: `AnalysisPageResult[]`

## Stream run events (SSE)

- `GET /api/runs/{run_id}/events`
- Server-sent events stream for real-time updates.
- Event types currently emitted:
  - `progress`
  - `page`
  - `complete`
  - `failed`
- Data payload is JSON for a `RunEvent`.

## List history

- `GET /api/history?limit=100&offset=0`
- Returns paginated run history ordered newest first.
- Query params:
  - `limit` (optional, default `100`, non-negative)
  - `offset` (optional, default `0`, non-negative)
- Response: `HistoryListItem[]`
