# pgAdmin-rs Setup Guide

## Quick Start

### Prerequisites
- Rust 1.75+ (install from [rustup.rs](https://rustup.rs/))
- PostgreSQL 12+ running on your system

### 1. Clone and Build

```bash
cd pgadmin-rs
cargo build --release
```

### 2. Create Environment File

```bash
cp .env.example .env
```

### 3. Configure Database Connection

Edit `.env` and update the PostgreSQL credentials:

```bash
SERVER_ADDRESS=0.0.0.0:3000
POSTGRES_HOST=localhost
POSTGRES_PORT=5432
POSTGRES_USER=postgres
POSTGRES_PASSWORD=your_password
POSTGRES_DB=postgres
RUST_LOG=info
```

### 4. Run the Application

```bash
cargo run
# or with release build:
./target/release/pgadmin-rs
```

The application will be available at: **http://localhost:3000**

## Common Setup Scenarios

### Using Docker PostgreSQL

If you have Docker installed and want to run PostgreSQL in a container:

```bash
docker run -d \
  --name pgadmin-postgres \
  -e POSTGRES_PASSWORD=mysecretpassword \
  -p 5432:5432 \
  postgres:15
```

Then update `.env`:
```bash
POSTGRES_PASSWORD=mysecretpassword
POSTGRES_DB=postgres
```

### Using Docker Compose

If you have the full `docker-compose.yml` set up:

```bash
docker-compose up -d
```

This will start PostgreSQL and the pgAdmin-rs application.

### Development with Hot Reload

Install `cargo-watch`:
```bash
cargo install cargo-watch
```

Then run with auto-reload on changes:
```bash
cargo watch -x run
```

## Troubleshooting

### "password authentication failed"

The PostgreSQL server is rejecting your credentials. Check:
1. PostgreSQL is running: `psql -U postgres`
2. Password is correct in `.env`
3. PostgreSQL user exists and has correct password

### "connection refused"

PostgreSQL is not running or not on the expected host/port:
1. Check PostgreSQL is running: `pg_isready -h localhost`
2. Verify host and port in `.env`
3. Check firewall isn't blocking port 5432

### Port 3000 already in use

Change the port in `.env`:
```bash
SERVER_ADDRESS=0.0.0.0:3001
```

### Build Errors

Ensure you have Rust 1.75+ installed:
```bash
rustc --version
rustup update
```

## Testing the Installation

Once running, visit http://localhost:3000 and you should see:

1. **Dashboard** - Welcome page with quick start guide
2. **Schema Browser** - Click to browse database schemas and tables
3. **Query Editor** - Enter SQL queries and see results

Try this sample query:
```sql
SELECT version();
```

## Next Steps

- Explore the **Schema Browser** to understand your database structure
- Try the **Query Editor** with sample SELECT queries
- Check **PROGRESS.md** for current features and roadmap
- Read **README.md** for architecture and feature details
