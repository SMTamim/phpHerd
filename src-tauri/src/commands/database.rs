use crate::core::service_manager::ServiceManager;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DbUser {
    pub username: String,
    pub host: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DbName {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateDbUserRequest {
    pub service_type: String,
    pub version: String,
    pub port: u16,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateDatabaseRequest {
    pub service_type: String,
    pub version: String,
    pub port: u16,
    pub db_name: String,
    pub owner: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DropDbUserRequest {
    pub service_type: String,
    pub version: String,
    pub port: u16,
    pub username: String,
}

/// Find the CLI client binary for a database service
fn find_client_binary(service_type: &str, version: &str) -> Result<std::path::PathBuf, String> {
    let bin_dir = ServiceManager::service_bin_dir(service_type, version);

    let candidates: Vec<&str> = match service_type {
        "mysql" => vec!["mysql.exe", "mysql"],
        "mariadb" => vec!["mariadb.exe", "mariadb", "mysql.exe", "mysql"],
        "postgresql" => vec!["psql.exe", "psql"],
        _ => return Err(format!("User management not supported for {}", service_type)),
    };

    for name in &candidates {
        let path = bin_dir.join(name);
        if path.exists() {
            return Ok(path);
        }
    }

    Err(format!(
        "CLI client not found for {} v{} in {:?}",
        service_type, version, bin_dir
    ))
}

/// Run a SQL command via the CLI client
fn run_sql(service_type: &str, version: &str, port: u16, sql: &str) -> Result<String, String> {
    let client = find_client_binary(service_type, version)?;

    let output = match service_type {
        "mysql" | "mariadb" => std::process::Command::new(&client)
            .args([
                "-u", "root",
                "-P", &port.to_string(),
                "-h", "127.0.0.1",
                "--skip-column-names",
                "-e", sql,
            ])
            .output()
            .map_err(|e| format!("Failed to run mysql client: {}", e))?,
        "postgresql" => std::process::Command::new(&client)
            .args([
                "-U", "postgres",
                "-p", &port.to_string(),
                "-h", "127.0.0.1",
                "-t", "-A",
                "-c", sql,
            ])
            .env("PGPASSWORD", "")
            .output()
            .map_err(|e| format!("Failed to run psql: {}", e))?,
        _ => return Err("Unsupported service type".to_string()),
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("SQL error: {}", stderr.trim()));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

#[tauri::command]
pub async fn list_db_users(
    service_type: String,
    version: String,
    port: u16,
) -> Result<Vec<DbUser>, String> {
    let sql = match service_type.as_str() {
        "mysql" | "mariadb" => "SELECT User, Host FROM mysql.user ORDER BY User;",
        "postgresql" => "SELECT usename, '127.0.0.1' FROM pg_catalog.pg_user ORDER BY usename;",
        _ => return Err(format!("User listing not supported for {}", service_type)),
    };

    let output = run_sql(&service_type, &version, port, sql)?;
    let mut users = Vec::new();

    for line in output.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let parts: Vec<&str> = match service_type.as_str() {
            "mysql" | "mariadb" => line.split('\t').collect(),
            "postgresql" => line.split('|').collect(),
            _ => continue,
        };
        if parts.len() >= 2 {
            users.push(DbUser {
                username: parts[0].trim().to_string(),
                host: parts[1].trim().to_string(),
            });
        }
    }

    Ok(users)
}

#[tauri::command]
pub async fn create_db_user(request: CreateDbUserRequest) -> Result<(), String> {
    match request.service_type.as_str() {
        "mysql" | "mariadb" => {
            let sql = format!(
                "CREATE USER IF NOT EXISTS '{}'@'localhost' IDENTIFIED BY '{}'; \
                 GRANT ALL PRIVILEGES ON *.* TO '{}'@'localhost' WITH GRANT OPTION; \
                 FLUSH PRIVILEGES;",
                request.username, request.password, request.username
            );
            run_sql(&request.service_type, &request.version, request.port, &sql)?;
        }
        "postgresql" => {
            // Check if user exists first
            let check = format!(
                "SELECT 1 FROM pg_roles WHERE rolname='{}'",
                request.username
            );
            let exists = run_sql(&request.service_type, &request.version, request.port, &check)
                .unwrap_or_default();

            if exists.trim() == "1" {
                // Update password
                let sql = format!(
                    "ALTER USER {} WITH PASSWORD '{}' CREATEDB;",
                    request.username, request.password
                );
                run_sql(&request.service_type, &request.version, request.port, &sql)?;
            } else {
                let sql = format!(
                    "CREATE USER {} WITH PASSWORD '{}' CREATEDB;",
                    request.username, request.password
                );
                run_sql(&request.service_type, &request.version, request.port, &sql)?;
            }
        }
        _ => return Err(format!("User creation not supported for {}", request.service_type)),
    }

    tracing::info!("Created database user '{}' on {} v{}", request.username, request.service_type, request.version);
    Ok(())
}

#[tauri::command]
pub async fn drop_db_user(request: DropDbUserRequest) -> Result<(), String> {
    match request.service_type.as_str() {
        "mysql" | "mariadb" => {
            let sql = format!(
                "DROP USER IF EXISTS '{}'@'localhost';",
                request.username
            );
            run_sql(&request.service_type, &request.version, request.port, &sql)?;
        }
        "postgresql" => {
            let sql = format!("DROP USER IF EXISTS {};", request.username);
            run_sql(&request.service_type, &request.version, request.port, &sql)?;
        }
        _ => return Err(format!("User deletion not supported for {}", request.service_type)),
    }

    tracing::info!("Dropped database user '{}'", request.username);
    Ok(())
}

#[tauri::command]
pub async fn list_databases(
    service_type: String,
    version: String,
    port: u16,
) -> Result<Vec<DbName>, String> {
    let sql = match service_type.as_str() {
        "mysql" | "mariadb" => "SHOW DATABASES;",
        "postgresql" => "SELECT datname FROM pg_database WHERE datistemplate = false ORDER BY datname;",
        _ => return Err(format!("Database listing not supported for {}", service_type)),
    };

    let output = run_sql(&service_type, &version, port, sql)?;
    let databases = output
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| DbName {
            name: l.trim().to_string(),
        })
        .collect();

    Ok(databases)
}

#[tauri::command]
pub async fn create_database(request: CreateDatabaseRequest) -> Result<(), String> {
    match request.service_type.as_str() {
        "mysql" | "mariadb" => {
            let sql = format!(
                "CREATE DATABASE IF NOT EXISTS `{}` CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;",
                request.db_name
            );
            run_sql(&request.service_type, &request.version, request.port, &sql)?;
        }
        "postgresql" => {
            let owner = request.owner.as_deref().unwrap_or("postgres");
            let sql = format!(
                "CREATE DATABASE \"{}\" OWNER \"{}\";",
                request.db_name, owner
            );
            run_sql(&request.service_type, &request.version, request.port, &sql)?;
        }
        _ => return Err(format!("Database creation not supported for {}", request.service_type)),
    }

    tracing::info!("Created database '{}'", request.db_name);
    Ok(())
}

#[tauri::command]
pub async fn drop_database(
    service_type: String,
    version: String,
    port: u16,
    db_name: String,
) -> Result<(), String> {
    match service_type.as_str() {
        "mysql" | "mariadb" => {
            let sql = format!("DROP DATABASE IF EXISTS `{}`;", db_name);
            run_sql(&service_type, &version, port, &sql)?;
        }
        "postgresql" => {
            let sql = format!("DROP DATABASE IF EXISTS \"{}\";", db_name);
            run_sql(&service_type, &version, port, &sql)?;
        }
        _ => return Err(format!("Database deletion not supported for {}", service_type)),
    }

    tracing::info!("Dropped database '{}'", db_name);
    Ok(())
}
