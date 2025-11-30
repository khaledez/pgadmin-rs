# Database Migrations

This directory contains database migration files for pgAdmin-rs.

## Usage

Migrations are managed using SQLx's built-in migration support.

### Creating a new migration

```bash
sqlx migrate add <migration_name>
```

### Running migrations

```bash
sqlx migrate run
```

### Reverting migrations

```bash
sqlx migrate revert
```

## Notes

- Migrations are applied in order based on their timestamp prefix
- Each migration should be atomic and reversible when possible
- Test migrations thoroughly before applying to production databases
