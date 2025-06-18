# AI-Powered User Management API

This example demonstrates an advanced Backworks configuration with:
- Multiple execution modes (mock, database, runtime)
- AI-powered features for pattern recognition and schema prediction
- Real-time dashboard monitoring
- Multi-runtime handlers (Node.js, Python)
- Database integration with automatic CRUD generation
- Smart API evolution from mock to production

## Quick Start

1. **Initialize the database** (SQLite for simplicity):
   ```bash
   sqlite3 users.db < schema.sql
   ```

2. **Start the API**:
   ```bash
   backworks start
   ```

3. **Open the dashboard**:
   Navigate to http://localhost:3001 to see real-time metrics and AI insights.

4. **Test the endpoints**:
   ```bash
   # Get all users (mock mode initially)
   curl http://localhost:3000/users
   
   # Create a user (switches to database mode)
   curl -X POST http://localhost:3000/users \
     -H "Content-Type: application/json" \
     -d '{"name": "Alice", "email": "alice@example.com"}'
   
   # Get AI-generated user recommendations (runtime handler)
   curl http://localhost:3000/users/recommendations
   ```

## Features Demonstrated

### 1. **Mode Evolution**
The API starts in mock mode and automatically evolves:
- `GET /users` → Mock responses initially
- `POST /users` → Switches to database mode when creating users
- `GET /users/recommendations` → Uses AI runtime handler

### 2. **AI Enhancements**
- **Pattern Recognition**: Tracks request patterns and suggests optimizations
- **Schema Prediction**: Analyzes request/response data to predict API schema
- **Anomaly Detection**: Identifies unusual request patterns or response times
- **Smart Recommendations**: AI-powered user recommendation engine

### 3. **Multi-Runtime Support**
- **Python Handler**: `handlers/recommendations.py` - ML-based user recommendations
- **Node.js Handler**: `handlers/analytics.js` - Real-time analytics processing

### 4. **Database Integration**
- Automatic CRUD operations for the `users` table
- Schema introspection and validation
- Query parameterization and safety

### 5. **Dashboard & Monitoring**
- Real-time request metrics
- Architecture visualization
- AI insights and suggestions
- Performance analytics

## Configuration Highlights

The `backworks.yaml` file demonstrates:
- Multi-mode endpoint definitions
- AI feature configuration
- Database connection setup
- Runtime handler registration
- Dashboard customization

## File Structure

```
.
├── backworks.yaml          # Main configuration
├── schema.sql             # Database schema
├── handlers/
│   ├── recommendations.py # Python AI handler
│   └── analytics.js       # Node.js analytics handler
└── README.md              # This file
```

## Evolution Flow

1. **Start**: All endpoints in mock mode
2. **Learn**: AI captures patterns from requests
3. **Evolve**: Endpoints switch to database/runtime based on usage
4. **Optimize**: AI suggests improvements and new endpoints
5. **Scale**: Dashboard shows performance metrics and bottlenecks
