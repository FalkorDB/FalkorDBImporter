# Development Setup

This document describes how to set up the development environment for the FalkorDB Data Importer.

## Prerequisites

- **Rust**: 1.75 or later
- **Node.js**: 20 or later
- **Docker**: For running FalkorDB locally
- **Docker Compose**: For orchestrating services

## Project Structure

```
.
├── backend/          # Rust backend with Axum
├── frontend/         # React/TypeScript frontend with Vite
├── shared/           # Shared TypeScript types
├── .github/          # GitHub Actions CI/CD
└── docker-compose.yml
```

## Quick Start

### 1. Clone the repository

```bash
git clone https://github.com/FalkorDB/FalkorDBImporter.git
cd FalkorDBImporter
```

### 2. Start FalkorDB with Docker

```bash
docker-compose up -d falkordb
```

### 3. Build and run with a single server

The backend now serves both the API and the frontend static files.

```bash
# Build the frontend
cd frontend
npm install
npm run build
cd ..

# Run the backend (serves both API and frontend)
cd backend
cargo run
```

The application will be available at `http://localhost:3000`:
- Frontend: `http://localhost:3000`
- Health check: `http://localhost:3000/health`
- Future API routes: `http://localhost:3000/api/*`

### 4. (Alternative) Development mode with separate servers

For frontend development with hot-reload:

```bash
# Terminal 1: Run the backend
cd backend
cargo run

# Terminal 2: Run the frontend dev server
cd frontend
npm run dev
```

In this mode, the frontend will be available at `http://localhost:5173` with hot-reload enabled.

## Development Commands

### Backend (Rust)

```bash
# Build
cargo build

# Run
cargo run

# Test
cargo test

# Format code
cargo fmt

# Lint with clippy
cargo clippy

# Check without building
cargo check
```

### Frontend (React/TypeScript)

```bash
# Install dependencies
npm install

# Run development server
npm run dev

# Build for production
npm run build

# Lint
npm run lint

# Preview production build
npm run preview
```

### Shared Types

```bash
cd shared

# Build types
npm run build

# Watch for changes
npm run watch
```

## Static File Serving

The Rust backend serves the frontend static files from `../frontend/dist`. This allows running a single server for both the API and the frontend.

**Key features:**
- API routes (like `/health` and future `/api/*` routes) are served first
- Static files from the frontend build are served for other requests
- SPA fallback: Unknown routes serve `index.html` to support client-side routing

**To rebuild the frontend:**
```bash
cd frontend
npm run build
```

The backend will automatically serve the updated files on the next request.

## Docker Development

To run the entire stack with Docker:

```bash
# Start all services
docker-compose up

# Start in detached mode
docker-compose up -d

# Stop all services
docker-compose down

# Rebuild images
docker-compose build

# View logs
docker-compose logs -f
```

## Code Quality

### Rust

- **rustfmt**: Automatic code formatting
  ```bash
  cargo fmt --all
  ```

- **clippy**: Linting and code analysis
  ```bash
  cargo clippy --all-targets -- -D warnings
  ```

### TypeScript

- **ESLint**: Linting
  ```bash
  cd frontend
  npm run lint
  ```

## CI/CD

GitHub Actions automatically:
- Checks code formatting (rustfmt)
- Runs linter (clippy)
- Runs all tests
- Builds the project

See `.github/workflows/rust-ci.yml` for details.

## Environment Variables

### Backend

- `RUST_LOG`: Log level (default: `info`)
- `FALKORDB_HOST`: FalkorDB host (default: `localhost`)
- `FALKORDB_PORT`: FalkorDB port (default: `6379`)
- `FRONTEND_DIR`: Path to frontend build directory (default: `../frontend/dist`)

### Frontend

- `VITE_API_URL`: Backend API URL (default: `http://localhost:3000`)

## Troubleshooting

### Port conflicts

If ports 3000, 5173, or 6379 are already in use, you can modify them in:
- Backend: Update the port in `backend/src/main.rs`
- Frontend: Update the port in `frontend/vite.config.ts`
- FalkorDB: Update the port mapping in `docker-compose.yml`

### Cargo build issues

```bash
# Clean build artifacts
cargo clean

# Update dependencies
cargo update
```

### Node.js issues

```bash
# Clean node_modules
rm -rf node_modules package-lock.json
npm install
```

## Additional Resources

- [Rust Documentation](https://doc.rust-lang.org/)
- [Axum Documentation](https://docs.rs/axum/)
- [React Documentation](https://react.dev/)
- [Vite Documentation](https://vitejs.dev/)
- [FalkorDB Documentation](https://docs.falkordb.com/)
