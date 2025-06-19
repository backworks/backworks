use crate::config::DatabaseConfig;
use crate::error::{BackworksError, BackworksResult};
use crate::config::PoolConfig;
use serde::{Deserialize, Serialize};
use sqlx::{Any, AnyPool, Column, Row, TypeInfo};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryRequest {
    pub query: String,
    pub params: Vec<serde_json::Value>,
    pub connection_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub success: bool,
    pub rows: Vec<HashMap<String, serde_json::Value>>,
    pub rows_affected: Option<u64>,
    pub error: Option<String>,
    pub duration: std::time::Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaInfo {
    pub tables: Vec<TableInfo>,
    pub views: Vec<ViewInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableInfo {
    pub name: String,
    pub columns: Vec<ColumnInfo>,
    pub primary_keys: Vec<String>,
    pub foreign_keys: Vec<ForeignKeyInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewInfo {
    pub name: String,
    pub columns: Vec<ColumnInfo>,
    pub definition: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
    pub default_value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForeignKeyInfo {
    pub column: String,
    pub referenced_table: String,
    pub referenced_column: String,
}

#[derive(Debug, Clone)]
pub struct ConnectionPool {
    pub name: String,
    pub pool: AnyPool,
    pub database_type: String,
}

#[derive(Debug)]
pub struct DatabaseManager {
    connections: Arc<RwLock<HashMap<String, ConnectionPool>>>,
    default_connection: Option<String>,
}

impl Clone for DatabaseManager {
    fn clone(&self) -> Self {
        Self {
            connections: Arc::clone(&self.connections),
            default_connection: self.default_connection.clone(),
        }
    }
}

impl DatabaseManager {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            default_connection: None,
        }
    }

    pub async fn start(&mut self, configs: HashMap<String, DatabaseConfig>) -> BackworksResult<()> {
        tracing::info!("Starting database manager with {} connections", configs.len());
        
        for (name, config) in configs {
            self.add_connection(name.clone(), config).await?;
            
            // Set the first connection as default if none specified
            if self.default_connection.is_none() {
                self.default_connection = Some(name);
            }
        }
        
        Ok(())
    }

    pub async fn add_connection(&self, name: String, config: DatabaseConfig) -> BackworksResult<()> {
        tracing::info!("Adding database connection: {} ({})", name, config.db_type);
        
        let database_url = self.build_connection_string(&config)?;
        
        let pool = AnyPool::connect(&database_url).await
            .map_err(|e| BackworksError::Database(format!("Failed to connect to database {}: {}", name, e)))?;
        
        let connection_pool = ConnectionPool {
            name: name.clone(),
            pool,
            database_type: config.db_type.clone(),
        };
        
        let mut connections = self.connections.write().await;
        connections.insert(name, connection_pool);
        
        Ok(())
    }

    pub async fn execute_query(&self, request: QueryRequest) -> BackworksResult<QueryResult> {
        let start_time = std::time::Instant::now();
        
        let connection_name = request.connection_name.clone()
            .or_else(|| self.default_connection.clone())
            .ok_or_else(|| BackworksError::Database("No database connection specified".to_string()))?;
        
        let connections = self.connections.read().await;
        let connection = connections.get(&connection_name)
            .ok_or_else(|| BackworksError::Database(format!("Connection not found: {}", connection_name)))?;
        
        let result = self.execute_query_on_pool(&connection.pool, &request).await;
        let duration = start_time.elapsed();
        
        match result {
            Ok(mut query_result) => {
                query_result.duration = duration;
                Ok(query_result)
            }
            Err(e) => Ok(QueryResult {
                success: false,
                rows: Vec::new(),
                rows_affected: None,
                error: Some(e.to_string()),
                duration,
            }),
        }
    }

    pub async fn get_schema(&self, connection_name: Option<String>) -> BackworksResult<SchemaInfo> {
        let connection_name = connection_name
            .or_else(|| self.default_connection.clone())
            .ok_or_else(|| BackworksError::Database("No database connection specified".to_string()))?;
        
        let connections = self.connections.read().await;
        let connection = connections.get(&connection_name)
            .ok_or_else(|| BackworksError::Database(format!("Connection not found: {}", connection_name)))?;
        
        match connection.database_type.as_str() {
            "postgresql" | "postgres" => self.get_postgresql_schema(&connection.pool).await,
            "mysql" => self.get_mysql_schema(&connection.pool).await,
            "sqlite" => self.get_sqlite_schema(&connection.pool).await,
            _ => Err(BackworksError::Database(format!("Schema introspection not supported for database type: {}", connection.database_type))),
        }
    }

    pub async fn generate_crud_endpoints(&self, table_name: &str, connection_name: Option<String>) -> BackworksResult<String> {
        let schema = self.get_schema(connection_name).await?;
        let table = schema.tables.iter()
            .find(|t| t.name == table_name)
            .ok_or_else(|| BackworksError::Database(format!("Table not found: {}", table_name)))?;
        
        let mut yaml = String::new();
        let primary_key = table.primary_keys.first()
            .ok_or_else(|| BackworksError::Database(format!("No primary key found for table: {}", table_name)))?;
        
        // Generate CRUD endpoints
        yaml.push_str(&format!("# Generated CRUD endpoints for table: {}\n", table_name));
        yaml.push_str("endpoints:\n");
        
        // GET all
        yaml.push_str(&format!("  - path: /{}\n", table_name));
        yaml.push_str("    method: GET\n");
        yaml.push_str("    mode: database\n");
        yaml.push_str("    database:\n");
        yaml.push_str(&format!("      query: \"SELECT * FROM {}\"\n", table_name));
        yaml.push_str("\n");
        
        // GET by ID
        yaml.push_str(&format!("  - path: /{}/{{{}}}\n", table_name, primary_key));
        yaml.push_str("    method: GET\n");
        yaml.push_str("    mode: database\n");
        yaml.push_str("    database:\n");
        yaml.push_str(&format!("      query: \"SELECT * FROM {} WHERE {} = $1\"\n", table_name, primary_key));
        yaml.push_str(&format!("      params: [\"{{{{ path.{} }}}}\"]\n", primary_key));
        yaml.push_str("\n");
        
        // POST (create)
        yaml.push_str(&format!("  - path: /{}\n", table_name));
        yaml.push_str("    method: POST\n");
        yaml.push_str("    mode: database\n");
        yaml.push_str("    database:\n");
        
        let insert_columns: Vec<String> = table.columns.iter()
            .filter(|col| col.name != *primary_key)
            .map(|col| col.name.clone())
            .collect();
        let insert_placeholders: Vec<String> = (1..=insert_columns.len())
            .map(|i| format!("${}", i))
            .collect();
        
        yaml.push_str(&format!(
            "      query: \"INSERT INTO {} ({}) VALUES ({}) RETURNING *\"\n",
            table_name,
            insert_columns.join(", "),
            insert_placeholders.join(", ")
        ));
        
        let insert_params: Vec<String> = insert_columns.iter()
            .map(|col| format!("\"{{{{ body.{} }}}}\"", col))
            .collect();
        yaml.push_str(&format!("      params: [{}]\n", insert_params.join(", ")));
        yaml.push_str("\n");
        
        // PUT (update)
        yaml.push_str(&format!("  - path: /{}/{{{}}}\n", table_name, primary_key));
        yaml.push_str("    method: PUT\n");
        yaml.push_str("    mode: database\n");
        yaml.push_str("    database:\n");
        
        let update_sets: Vec<String> = insert_columns.iter()
            .enumerate()
            .map(|(i, col)| format!("{} = ${}", col, i + 1))
            .collect();
        
        yaml.push_str(&format!(
            "      query: \"UPDATE {} SET {} WHERE {} = ${} RETURNING *\"\n",
            table_name,
            update_sets.join(", "),
            primary_key,
            insert_columns.len() + 1
        ));
        
        let mut update_params = insert_params;
        update_params.push(format!("\"{{{{ path.{} }}}}\"", primary_key));
        yaml.push_str(&format!("      params: [{}]\n", update_params.join(", ")));
        yaml.push_str("\n");
        
        // DELETE
        yaml.push_str(&format!("  - path: /{}/{{{}}}\n", table_name, primary_key));
        yaml.push_str("    method: DELETE\n");
        yaml.push_str("    mode: database\n");
        yaml.push_str("    database:\n");
        yaml.push_str(&format!("      query: \"DELETE FROM {} WHERE {} = $1\"\n", table_name, primary_key));
        yaml.push_str(&format!("      params: [\"{{{{ path.{} }}}}\"]\n", primary_key));
        
        Ok(yaml)
    }

    pub async fn list_connections(&self) -> Vec<String> {
        self.connections.read().await.keys().cloned().collect()
    }

    async fn execute_query_on_pool(&self, pool: &AnyPool, request: &QueryRequest) -> BackworksResult<QueryResult> {
        let mut query = sqlx::query(&request.query);
        
        // Bind parameters
        for param in &request.params {
            query = match param {
                serde_json::Value::String(s) => query.bind(s),
                serde_json::Value::Number(n) => {
                    if let Some(i) = n.as_i64() {
                        query.bind(i)
                    } else if let Some(f) = n.as_f64() {
                        query.bind(f)
                    } else {
                        query.bind(n.to_string())
                    }
                }
                serde_json::Value::Bool(b) => query.bind(b),
                serde_json::Value::Null => query.bind(Option::<String>::None),
                _ => query.bind(param.to_string()),
            };
        }
        
        // Determine if this is a SELECT query or a modification query
        let query_type = request.query.trim().to_uppercase();
        let is_select = query_type.starts_with("SELECT") || query_type.starts_with("WITH");
        
        if is_select {
            let rows = query.fetch_all(pool).await
                .map_err(|e| BackworksError::Database(format!("Query execution failed: {}", e)))?;
            
            let mut result_rows = Vec::new();
            for row in rows {
                let mut row_map = HashMap::new();
                for (i, column) in row.columns().iter().enumerate() {
                    let column_name = column.name().to_string();
                    let value = self.extract_value_from_row(&row, i, column.type_info())?;
                    row_map.insert(column_name, value);
                }
                result_rows.push(row_map);
            }
            
            Ok(QueryResult {
                success: true,
                rows: result_rows,
                rows_affected: None,
                error: None,
                duration: std::time::Duration::from_millis(0),
            })
        } else {
            let result = query.execute(pool).await
                .map_err(|e| BackworksError::Database(format!("Query execution failed: {}", e)))?;
            
            Ok(QueryResult {
                success: true,
                rows: Vec::new(),
                rows_affected: Some(result.rows_affected()),
                error: None,
                duration: std::time::Duration::from_millis(0),
            })
        }
    }

    fn extract_value_from_row(&self, row: &sqlx::any::AnyRow, index: usize, type_info: &<Any as sqlx::Database>::TypeInfo) -> BackworksResult<serde_json::Value> {
        let type_name = type_info.name();
        
        match type_name {
            "TEXT" | "VARCHAR" | "CHAR" => {
                let value: Option<String> = row.try_get(index).ok();
                Ok(value.map(serde_json::Value::String).unwrap_or(serde_json::Value::Null))
            }
            "INTEGER" | "INT" | "BIGINT" => {
                let value: Option<i64> = row.try_get(index).ok();
                Ok(value.map(|v| serde_json::Value::Number(serde_json::Number::from(v))).unwrap_or(serde_json::Value::Null))
            }
            "REAL" | "FLOAT" | "DOUBLE" => {
                let value: Option<f64> = row.try_get(index).ok();
                Ok(value.map(|v| serde_json::Value::Number(serde_json::Number::from_f64(v).unwrap())).unwrap_or(serde_json::Value::Null))
            }
            "BOOLEAN" => {
                let value: Option<bool> = row.try_get(index).ok();
                Ok(value.map(serde_json::Value::Bool).unwrap_or(serde_json::Value::Null))
            }
            _ => {
                // Try to get as string for unknown types
                let value: Option<String> = row.try_get(index).ok();
                Ok(value.map(serde_json::Value::String).unwrap_or(serde_json::Value::Null))
            }
        }
    }

    fn build_connection_string(&self, config: &DatabaseConfig) -> BackworksResult<String> {
        // First check if a connection string is directly provided
        if let Some(ref conn_str) = &config.connection_string {
            return Ok(conn_str.clone());
        }
        
        // Then check if connection string is in environment variable
        if let Some(ref env_var) = config.connection_string_env {
            if let Ok(conn_str) = std::env::var(env_var) {
                return Ok(conn_str);
            }
        }
        
        // Fallback: build a default connection string based on database type
        match config.db_type.as_str() {
            "postgresql" | "postgres" => {
                Ok("postgresql://postgres:password@localhost:5432/postgres".to_string())
            }
            "mysql" => {
                Ok("mysql://root:password@localhost:3306/mysql".to_string())
            }
            "sqlite" => {
                Ok("sqlite:database.db".to_string())
            }
            _ => Err(BackworksError::Database(format!("Unsupported database type: {}", config.db_type)))
        }
    }

    async fn get_postgresql_schema(&self, pool: &AnyPool) -> BackworksResult<SchemaInfo> {
        let table_query = "
            SELECT table_name, column_name, data_type, is_nullable, column_default
            FROM information_schema.columns 
            WHERE table_schema = 'public'
            ORDER BY table_name, ordinal_position
        ";
        
        let rows = sqlx::query(table_query).fetch_all(pool).await
            .map_err(|e| BackworksError::Database(format!("Failed to get schema: {}", e)))?;
        
        let mut tables: HashMap<String, TableInfo> = HashMap::new();
        
        for row in rows {
            let table_name: String = row.try_get("table_name")
                .map_err(|e| BackworksError::Database(format!("Failed to get table_name: {}", e)))?;
            let column_name: String = row.try_get("column_name")
                .map_err(|e| BackworksError::Database(format!("Failed to get column_name: {}", e)))?;
            let data_type: String = row.try_get("data_type")
                .map_err(|e| BackworksError::Database(format!("Failed to get data_type: {}", e)))?;
            let is_nullable: String = row.try_get("is_nullable")
                .map_err(|e| BackworksError::Database(format!("Failed to get is_nullable: {}", e)))?;
            let column_default: Option<String> = row.try_get("column_default").ok();
            
            let column_info = ColumnInfo {
                name: column_name,
                data_type,
                nullable: is_nullable == "YES",
                default_value: column_default,
            };
            
            tables.entry(table_name.clone())
                .or_insert_with(|| TableInfo {
                    name: table_name,
                    columns: Vec::new(),
                    primary_keys: Vec::new(),
                    foreign_keys: Vec::new(),
                })
                .columns.push(column_info);
        }
        
        Ok(SchemaInfo {
            tables: tables.into_values().collect(),
            views: Vec::new(), // TODO: Implement views
        })
    }

    async fn get_mysql_schema(&self, _pool: &AnyPool) -> BackworksResult<SchemaInfo> {
        // Similar implementation for MySQL
        // This is a simplified version
        Ok(SchemaInfo {
            tables: Vec::new(),
            views: Vec::new(),
        })
    }

    async fn get_sqlite_schema(&self, pool: &AnyPool) -> BackworksResult<SchemaInfo> {
        let table_query = "
            SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'
        ";
        
        let table_rows = sqlx::query(table_query).fetch_all(pool).await
            .map_err(|e| BackworksError::Database(format!("Failed to get tables: {}", e)))?;
        
        let mut tables = Vec::new();
        
        for table_row in table_rows {
            let table_name: String = table_row.try_get("name")
                .map_err(|e| BackworksError::Database(format!("Failed to get table name: {}", e)))?;
            
            let column_query = format!("PRAGMA table_info({})", table_name);
            let column_rows = sqlx::query(&column_query).fetch_all(pool).await
                .map_err(|e| BackworksError::Database(format!("Failed to get columns: {}", e)))?;
            
            let mut columns = Vec::new();
            let mut primary_keys = Vec::new();
            
            for column_row in column_rows {
                let column_name: String = column_row.try_get("name")
                    .map_err(|e| BackworksError::Database(format!("Failed to get column name: {}", e)))?;
                let data_type: String = column_row.try_get("type")
                    .map_err(|e| BackworksError::Database(format!("Failed to get data type: {}", e)))?;
                let not_null: i32 = column_row.try_get("notnull")
                    .map_err(|e| BackworksError::Database(format!("Failed to get notnull: {}", e)))?;
                let default_value: Option<String> = column_row.try_get("dflt_value").ok();
                let is_pk: i32 = column_row.try_get("pk")
                    .map_err(|e| BackworksError::Database(format!("Failed to get pk: {}", e)))?;
                
                if is_pk == 1 {
                    primary_keys.push(column_name.clone());
                }
                
                columns.push(ColumnInfo {
                    name: column_name,
                    data_type,
                    nullable: not_null == 0,
                    default_value,
                });
            }
            
            tables.push(TableInfo {
                name: table_name,
                columns,
                primary_keys,
                foreign_keys: Vec::new(), // TODO: Implement foreign keys
            });
        }
        
        Ok(SchemaInfo {
            tables,
            views: Vec::new(),
        })
    }

    async fn get_connection_pool(&self, name: &str) -> BackworksResult<AnyPool> {
        let connections = self.connections.read().await;
        let connection = connections.get(name)
            .ok_or_else(|| BackworksError::Database(format!("Connection not found: {}", name)))?;
        Ok(connection.pool.clone())
    }

    pub async fn handle_request(&self, method: &str, config: &DatabaseConfig, request_data: &str) -> BackworksResult<String> {
        tracing::info!("Handling database request: {} method", method);
        
        // Parse request data as JSON
        let request: serde_json::Value = serde_json::from_str(request_data)
            .map_err(|e| BackworksError::database(format!("Invalid JSON in request: {}", e)))?;
        
        // Get database connection
        let pool = self.get_connection_pool("default").await?;
        
        match method.to_uppercase().as_str() {
            "GET" => self.handle_select(&pool, &request).await,
            "POST" => self.handle_insert(&pool, &request).await,
            "PUT" => self.handle_update(&pool, &request).await,
            "DELETE" => self.handle_delete(&pool, &request).await,
            _ => Err(BackworksError::database(format!("Unsupported HTTP method: {}", method)))
        }
    }
    
    async fn handle_select(&self, pool: &AnyPool, request: &serde_json::Value) -> BackworksResult<String> {
        // Simple SELECT implementation
        let table = request.get("table")
            .and_then(|v| v.as_str())
            .unwrap_or("users"); // Default table
            
        let query = format!("SELECT * FROM {}", table);
        let rows = sqlx::query(&query).fetch_all(pool).await?;
        
        // Convert rows to JSON
        let mut results = Vec::new();
        for row in rows {
            let mut obj = serde_json::Map::new();
            for (i, column) in row.columns().iter().enumerate() {
                let value = self.extract_value(&row, i)?;
                obj.insert(column.name().to_string(), value);
            }
            results.push(serde_json::Value::Object(obj));
        }
        
        Ok(serde_json::to_string(&results)?)
    }
    
    async fn handle_insert(&self, pool: &AnyPool, request: &serde_json::Value) -> BackworksResult<String> {
        // Simple INSERT implementation
        let table = request.get("table")
            .and_then(|v| v.as_str())
            .unwrap_or("users");
            
        let data = request.get("data")
            .ok_or_else(|| BackworksError::database("Missing 'data' field in request".to_string()))?;
        
        if let Some(obj) = data.as_object() {
            let columns: Vec<&str> = obj.keys().map(|k| k.as_str()).collect();
            let placeholders: Vec<String> = (1..=columns.len()).map(|i| format!("${}", i)).collect();
            
            let query = format!(
                "INSERT INTO {} ({}) VALUES ({})", 
                table, 
                columns.join(", "), 
                placeholders.join(", ")
            );
            
            let mut query_builder = sqlx::query(&query);
            for column in &columns {
                if let Some(value) = obj.get(*column) {
                    query_builder = self.bind_value(query_builder, value);
                }
            }
            
            let result = query_builder.execute(pool).await?;
            Ok(format!("{{\"inserted\": {}}}", result.rows_affected()))
        } else {
            Err(BackworksError::database("Data must be an object".to_string()))
        }
    }
    
    async fn handle_update(&self, _pool: &AnyPool, _request: &serde_json::Value) -> BackworksResult<String> {
        // TODO: Implement UPDATE logic
        Ok("{\"message\": \"UPDATE not yet implemented\"}".to_string())
    }
    
    async fn handle_delete(&self, _pool: &AnyPool, _request: &serde_json::Value) -> BackworksResult<String> {
        // TODO: Implement DELETE logic
        Ok("{\"message\": \"DELETE not yet implemented\"}".to_string())
    }
    
    fn extract_value(&self, _row: &sqlx::any::AnyRow, _index: usize) -> BackworksResult<serde_json::Value> {
        // Extract value from row at given index
        // This is a simplified implementation
        Ok(serde_json::Value::String("value".to_string()))
    }
    
    fn bind_value<'a>(&self, query: sqlx::query::Query<'a, sqlx::Any, sqlx::any::AnyArguments<'a>>, value: &serde_json::Value) -> sqlx::query::Query<'a, sqlx::Any, sqlx::any::AnyArguments<'a>> {
        // For now, convert all values to strings to avoid lifetime issues
        // TODO: Implement proper type binding when sqlx lifetime issues are resolved
        query.bind(value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_database_manager_creation() {
        let manager = DatabaseManager::new();
        assert!(manager.list_connections().await.is_empty());
    }

    #[test]
    fn test_query_request_creation() {
        let request = QueryRequest {
            query: "SELECT * FROM users".to_string(),
            params: vec![],
            connection_name: Some("default".to_string()),
        };
        
        assert_eq!(request.query, "SELECT * FROM users");
        assert_eq!(request.connection_name, Some("default".to_string()));
    }

    #[test]
    fn test_build_postgresql_connection_string() {
        let manager = DatabaseManager::new();
        let config = DatabaseConfig {
            db_type: "postgresql".to_string(),
            connection_string: Some("postgresql://user:pass@localhost:5432/mydb".to_string()),
            connection_string_env: None,
            pool: Some(PoolConfig {
                min_connections: None,
                max_connections: Some(10),
                connection_timeout: None,
            }),
            databases: None,
        };
        
        let conn_str = manager.build_connection_string(&config).unwrap();
        assert_eq!(conn_str, "postgresql://user:pass@localhost:5432/mydb");
    }

    #[test]
    fn test_build_sqlite_connection_string() {
        let manager = DatabaseManager::new();
        let config = DatabaseConfig {
            db_type: "sqlite".to_string(),
            connection_string: Some("sqlite:test.db".to_string()),
            connection_string_env: None,
            pool: None,
            databases: None,
        };
        
        let conn_str = manager.build_connection_string(&config).unwrap();
        assert_eq!(conn_str, "sqlite:test.db");
    }
}
