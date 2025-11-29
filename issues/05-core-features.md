# Issue #05: Core Features Implementation

## Overview
Implement the main features of pgAdmin-rs including database browsing, query execution, table data management, and schema operations.

## Goals
- Browse databases, schemas, and tables
- Execute SQL queries with results display
- View and edit table data
- Manage database objects
- Export query results

## Features Breakdown

### 1. Dashboard / Home Page

**Overview page showing:**
- Connected database information
- Database size and statistics
- Quick actions (new query, browse tables, etc.)
- Recent queries history
- Active connections count

```rust
// src/routes/dashboard.rs
pub async fn dashboard(
    State(state): State<AppState>,
    session: Session,
) -> Result<impl IntoResponse> {
    let db_info = state.db_service.get_database_info().await?;
    let stats = state.db_service.get_statistics().await?;
    let recent_queries = state.query_service.get_recent_queries(&session).await?;

    let template = DashboardTemplate {
        db_info,
        stats,
        recent_queries,
    };

    Ok(template.render()?)
}
```

### 2. Database Browser

**Hierarchical navigation:**
- Databases
  - Schemas
    - Tables
    - Views
    - Functions
    - Sequences

**Implementation:**
```rust
// src/routes/browser.rs
pub async fn browse_databases(
    State(state): State<AppState>,
) -> Result<impl IntoResponse> {
    let databases = state.db_service.list_databases().await?;

    Ok(DatabaseListTemplate { databases }.render()?)
}

pub async fn browse_schemas(
    Path(database): Path<String>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse> {
    let schemas = state.db_service.list_schemas(&database).await?;

    Ok(SchemaListTemplate { database, schemas }.render()?)
}

pub async fn browse_tables(
    Path((schema, table_type)): Path<(String, String)>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse> {
    let tables = state.db_service.list_tables(&schema, &table_type).await?;

    Ok(TableListTemplate { schema, tables }.render()?)
}
```

**HTMX integration for dynamic loading:**
```html
<!-- Clicking a schema loads its tables dynamically -->
<div hx-get="/browser/schema/{{ schema_name }}/tables"
     hx-trigger="click"
     hx-target="#table-list">
    {{ schema_name }}
</div>
```

### 3. SQL Query Editor

**Features:**
- Syntax highlighting (via JavaScript library like CodeMirror or Monaco)
- Query execution
- Results display in table format
- Query history
- Export results (CSV, JSON)
- Multiple query execution (separated by semicolons)

**Implementation:**
```rust
// src/routes/query.rs
pub async fn query_page(
    session: Session,
) -> Result<impl IntoResponse> {
    let history = get_query_history(&session).await?;

    Ok(QueryEditorTemplate { history }.render()?)
}

pub async fn execute_query(
    State(state): State<AppState>,
    session: Session,
    Form(form): Form<QueryForm>,
) -> Result<impl IntoResponse> {
    // Validate query
    state.validator.validate(&form.query)?;

    // Check if dangerous
    if state.validator.is_dangerous(&form.query) {
        return Ok(ConfirmationTemplate {
            query: form.query,
            warning: "This query may modify data. Confirm execution?",
        }.render()?);
    }

    // Execute query
    let start = Instant::now();
    let result = state.query_service.execute(&form.query).await?;
    let duration = start.elapsed();

    // Save to history
    save_to_history(&session, &form.query, duration).await?;

    // Return results
    Ok(QueryResultTemplate {
        columns: result.columns,
        rows: result.rows,
        row_count: result.row_count,
        duration,
    }.render()?)
}
```

**Query result format:**
```rust
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<serde_json::Value>>,
    pub row_count: usize,
    pub affected_rows: Option<u64>,
}
```

### 4. Table Data Viewer

**Features:**
- Paginated table data display
- Sort by column
- Filter/search
- Edit inline
- Delete rows
- Insert new rows
- Export table data

**Implementation:**
```rust
// src/routes/table_data.rs
pub async fn view_table_data(
    Path((schema, table)): Path<(String, String)>,
    Query(params): Query<TableDataParams>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse> {
    // Get table metadata
    let table_info = state.db_service.get_table_info(&schema, &table).await?;

    // Get paginated data
    let data = state.db_service.get_table_data(
        &schema,
        &table,
        params.page.unwrap_or(1),
        params.page_size.unwrap_or(100),
        params.sort_column.as_deref(),
        params.sort_direction.as_deref(),
    ).await?;

    Ok(TableDataTemplate {
        schema,
        table,
        table_info,
        data,
        pagination: calculate_pagination(&data),
    }.render()?)
}

#[derive(Deserialize)]
pub struct TableDataParams {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
    pub sort_column: Option<String>,
    pub sort_direction: Option<String>,
    pub filter: Option<String>,
}
```

**HTMX for pagination:**
```html
<div id="table-data">
    <table>
        <!-- Table content -->
    </table>

    <div class="pagination">
        <button hx-get="/table/{{ schema }}/{{ table }}?page={{ prev_page }}"
                hx-target="#table-data"
                hx-swap="outerHTML">
            Previous
        </button>
        <button hx-get="/table/{{ schema }}/{{ table }}?page={{ next_page }}"
                hx-target="#table-data"
                hx-swap="outerHTML">
            Next
        </button>
    </div>
</div>
```

### 5. Table Data Editing

**Inline editing with HTMX:**
```html
<!-- View mode -->
<td hx-get="/table/{{ schema }}/{{ table }}/cell/{{ row_id }}/{{ column }}/edit"
    hx-trigger="dblclick"
    hx-target="this"
    hx-swap="outerHTML">
    {{ value }}
</td>

<!-- Edit mode -->
<td>
    <input type="text"
           value="{{ value }}"
           hx-post="/table/{{ schema }}/{{ table }}/cell/{{ row_id }}/{{ column }}"
           hx-trigger="blur, keyup[key=='Enter']"
           hx-target="this"
           hx-swap="outerHTML">
</td>
```

**Backend handler:**
```rust
pub async fn update_cell(
    Path((schema, table, row_id, column)): Path<(String, String, String, String)>,
    State(state): State<AppState>,
    Form(form): Form<CellUpdateForm>,
) -> Result<impl IntoResponse> {
    // Validate input
    state.validator.validate_value(&form.value, &column)?;

    // Update cell
    state.db_service.update_cell(
        &schema,
        &table,
        &row_id,
        &column,
        &form.value,
    ).await?;

    // Return updated cell in view mode
    Ok(CellViewTemplate {
        value: form.value,
        schema,
        table,
        row_id,
        column,
    }.render()?)
}
```

### 6. Table Structure Viewer

**Display table information:**
- Columns (name, type, nullable, default, constraints)
- Indexes
- Foreign keys
- Triggers
- Table size and row count

```rust
pub async fn view_table_structure(
    Path((schema, table)): Path<(String, String)>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse> {
    let structure = state.db_service.get_table_structure(&schema, &table).await?;

    Ok(TableStructureTemplate { structure }.render()?)
}
```

### 7. Export Functionality

**Export query results or table data:**

```rust
// src/services/export_service.rs
pub struct ExportService;

impl ExportService {
    pub async fn export_to_csv(
        &self,
        columns: &[String],
        rows: &[Vec<serde_json::Value>],
    ) -> Result<String> {
        let mut wtr = csv::Writer::from_writer(vec![]);

        // Write headers
        wtr.write_record(columns)?;

        // Write rows
        for row in rows {
            let row_strings: Vec<String> = row
                .iter()
                .map(|v| value_to_string(v))
                .collect();
            wtr.write_record(&row_strings)?;
        }

        Ok(String::from_utf8(wtr.into_inner()?)?)
    }

    pub fn export_to_json(
        &self,
        columns: &[String],
        rows: &[Vec<serde_json::Value>],
    ) -> Result<String> {
        let objects: Vec<serde_json::Map<String, serde_json::Value>> = rows
            .iter()
            .map(|row| {
                let mut obj = serde_json::Map::new();
                for (i, col) in columns.iter().enumerate() {
                    obj.insert(col.clone(), row[i].clone());
                }
                obj
            })
            .collect();

        Ok(serde_json::to_string_pretty(&objects)?)
    }

    pub fn export_to_sql(
        &self,
        schema: &str,
        table: &str,
        columns: &[String],
        rows: &[Vec<serde_json::Value>],
    ) -> Result<String> {
        let mut sql = String::new();

        for row in rows {
            let values: Vec<String> = row
                .iter()
                .map(|v| format_value_for_sql(v))
                .collect();

            sql.push_str(&format!(
                "INSERT INTO {}.{} ({}) VALUES ({});\n",
                schema,
                table,
                columns.join(", "),
                values.join(", ")
            ));
        }

        Ok(sql)
    }
}
```

**Export endpoint:**
```rust
pub async fn export_data(
    Query(params): Query<ExportParams>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse> {
    let data = state.db_service.get_export_data(&params).await?;

    let content = match params.format.as_str() {
        "csv" => state.export_service.export_to_csv(&data.columns, &data.rows)?,
        "json" => state.export_service.export_to_json(&data.columns, &data.rows)?,
        "sql" => state.export_service.export_to_sql(
            &params.schema,
            &params.table,
            &data.columns,
            &data.rows
        )?,
        _ => return Err(AppError::InvalidFormat),
    };

    let filename = format!("{}_{}.{}", params.schema, params.table, params.format);

    Ok(Response::builder()
        .header("Content-Type", get_content_type(&params.format))
        .header("Content-Disposition", format!("attachment; filename=\"{}\"", filename))
        .body(content.into())?)
}
```

### 8. Schema Operations

**Create/modify/delete database objects:**

```rust
// src/routes/schema_ops.rs
pub async fn create_table_form(
    Path(schema): Path<String>,
) -> Result<impl IntoResponse> {
    Ok(CreateTableFormTemplate { schema }.render()?)
}

pub async fn create_table(
    Path(schema): Path<String>,
    State(state): State<AppState>,
    Form(form): Form<CreateTableForm>,
) -> Result<impl IntoResponse> {
    let sql = build_create_table_sql(&schema, &form)?;

    state.db_service.execute(&sql).await?;

    Ok(Redirect::to(&format!("/schema/{}/tables", schema)))
}

pub async fn drop_table(
    Path((schema, table)): Path<(String, String)>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse> {
    // This should require confirmation
    let sql = format!("DROP TABLE {}.{}", schema, table);

    state.db_service.execute(&sql).await?;

    Ok(Redirect::to(&format!("/schema/{}/tables", schema)))
}
```

### 9. Query History

**Track and display query history:**

```rust
// src/services/history_service.rs
pub struct QueryHistory {
    pub query: String,
    pub executed_at: DateTime<Utc>,
    pub duration: Duration,
    pub success: bool,
    pub row_count: Option<usize>,
}

pub async fn save_to_history(
    session: &Session,
    query: &str,
    duration: Duration,
    success: bool,
    row_count: Option<usize>,
) -> Result<()> {
    let mut history = session
        .get::<Vec<QueryHistory>>("query_history")
        .await?
        .unwrap_or_default();

    history.insert(0, QueryHistory {
        query: query.to_string(),
        executed_at: Utc::now(),
        duration,
        success,
        row_count,
    });

    // Keep only last 50 queries
    history.truncate(50);

    session.insert("query_history", history).await?;

    Ok(())
}
```

### 10. Database Statistics

**Display useful statistics:**
- Database size
- Table sizes
- Index sizes
- Row counts
- Bloat detection
- Slow queries (if pg_stat_statements available)

```rust
pub async fn get_statistics(&self) -> Result<DatabaseStats> {
    let db_size = sqlx::query_scalar!(
        "SELECT pg_database_size(current_database())"
    )
    .fetch_one(&self.pool)
    .await?;

    let table_count = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM information_schema.tables
         WHERE table_schema NOT IN ('pg_catalog', 'information_schema')"
    )
    .fetch_one(&self.pool)
    .await?;

    let largest_tables = sqlx::query!(
        "SELECT
            schemaname || '.' || tablename as table_name,
            pg_total_relation_size(schemaname || '.' || tablename) as size
         FROM pg_tables
         WHERE schemaname NOT IN ('pg_catalog', 'information_schema')
         ORDER BY size DESC
         LIMIT 10"
    )
    .fetch_all(&self.pool)
    .await?;

    Ok(DatabaseStats {
        db_size,
        table_count,
        largest_tables,
    })
}
```

## File Structure
```
src/
├── routes/
│   ├── dashboard.rs
│   ├── browser.rs
│   ├── query.rs
│   ├── table_data.rs
│   ├── table_structure.rs
│   ├── schema_ops.rs
│   └── export.rs
├── services/
│   ├── query_service.rs
│   ├── export_service.rs
│   ├── history_service.rs
│   └── stats_service.rs
├── templates/
│   ├── dashboard.html
│   ├── browser/
│   ├── query/
│   ├── table/
│   └── components/
```

## Testing Requirements
- [ ] Database browsing works correctly
- [ ] SQL queries execute and display results
- [ ] Table data pagination works
- [ ] Inline editing updates database
- [ ] Export formats generate correctly
- [ ] Query history tracks queries
- [ ] Statistics calculate accurately
- [ ] Schema operations work
- [ ] Error handling for invalid queries
- [ ] Large result sets handled efficiently

## Performance Considerations
- Paginate large result sets
- Limit query execution time
- Cache database metadata
- Lazy load tree structures
- Stream large exports

## Acceptance Criteria
- [ ] All core features implemented and working
- [ ] UI is responsive and functional
- [ ] Data operations are safe and validated
- [ ] Export functionality works for all formats
- [ ] Query history tracks executions
- [ ] Statistics display correctly
- [ ] Tests pass
- [ ] Documentation complete
