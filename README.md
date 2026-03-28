# ALICE-SLAM-SaaS

Simultaneous Localization and Mapping (SLAM) as a Service — real-time 3D map construction and robot localization via REST API.

## Architecture

```
Client
  |
  v
API Gateway (:8217)
  |
  v
Core Engine (:8117)
  |
  +-- SLAM Mapper
  +-- Loop Closure Detector
  +-- Map Exporter
```

## Features

- Real-time 3D occupancy grid construction from sensor streams
- 6-DoF pose estimation and localization
- Graph-based loop closure detection and correction
- Point cloud and mesh export (PLY, OBJ, PCD formats)
- Multi-session map merging
- Keyframe management with sliding window optimization

## API Endpoints

### Core Engine (port 8117)

| Method | Path | Description |
|--------|------|-------------|
| POST | /api/v1/slam/map | Submit sensor data to update the map |
| POST | /api/v1/slam/localize | Localize a query scan within the current map |
| POST | /api/v1/slam/loop-closure | Trigger loop closure detection and graph optimization |
| GET  | /api/v1/slam/export | Export the current map in the requested format |
| GET  | /api/v1/slam/stats | Return runtime statistics |
| GET  | /health | Health check |

### Example: Map Update

```bash
curl -X POST http://localhost:8117/api/v1/slam/map \
  -H 'Content-Type: application/json' \
  -d '{"session_id":"s1","scan":{"points":[[0.1,0.2,0.3]],"timestamp":1700000000}}'
```

### Example: Localize

```bash
curl -X POST http://localhost:8117/api/v1/slam/localize \
  -H 'Content-Type: application/json' \
  -d '{"session_id":"s1","scan":{"points":[[0.1,0.2,0.3]],"timestamp":1700000001}}'
```

## License

AGPL-3.0-or-later
