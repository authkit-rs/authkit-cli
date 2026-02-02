use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::cli::DatabaseType;
use crate::error::{CliError, CliResult};

/// AuthKit configuration file structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthKitConfig {
    /// Database configuration
    pub database: DatabaseConfig,

    /// Enabled features
    #[serde(default)]
    pub features: FeaturesConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Database type: "sqlite" or "postgres"
    #[serde(rename = "type")]
    pub db_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FeaturesConfig {
    /// Email/password authentication (always enabled, base feature)
    #[serde(default = "default_true")]
    pub email_password: bool,

    /// Email verification feature (adds email_verified columns to users)
    #[serde(default)]
    pub email_verification: bool,
    // Future features can be added here:
    // pub oauth: bool,
    // pub magic_link: bool,
    // pub two_factor: bool,
}

fn default_true() -> bool {
    true
}

impl AuthKitConfig {
    /// Load configuration from a TOML file
    pub fn load<P: AsRef<Path>>(path: P) -> CliResult<Self> {
        let path = path.as_ref();

        if !path.exists() {
            return Err(CliError::ConfigNotFound(path.display().to_string()));
        }

        let content = fs::read_to_string(path)?;
        let config: AuthKitConfig =
            toml::from_str(&content).map_err(|e| CliError::ConfigParse(e.to_string()))?;

        // Validate config
        config.validate()?;

        Ok(config)
    }

    /// Save configuration to a TOML file
    pub fn save<P: AsRef<Path>>(&self, path: P) -> CliResult<()> {
        let content =
            toml::to_string_pretty(self).map_err(|e| CliError::ConfigParse(e.to_string()))?;

        fs::write(path, content)?;
        Ok(())
    }

    /// Create a default configuration
    pub fn default_config(db_type: DatabaseType) -> Self {
        Self {
            database: DatabaseConfig {
                db_type: db_type.to_string(),
            },
            features: FeaturesConfig {
                email_password: true,
                email_verification: false,
            },
        }
    }

    /// Validate the configuration
    pub fn validate(&self) -> CliResult<()> {
        // Validate database type
        match self.database.db_type.as_str() {
            "sqlite" | "postgres" => {}
            other => {
                return Err(CliError::ConfigParse(format!(
                    "Invalid database type '{}'. Must be 'sqlite' or 'postgres'.",
                    other
                )));
            }
        }

        // email_password must always be enabled (it's the base)
        if !self.features.email_password {
            return Err(CliError::ConfigParse(
                "email_password feature must be enabled (it is the base feature)".to_string(),
            ));
        }

        Ok(())
    }

    /// Get the database type enum
    pub fn database_type(&self) -> CliResult<DatabaseType> {
        match self.database.db_type.as_str() {
            "sqlite" => Ok(DatabaseType::Sqlite),
            "postgres" => Ok(DatabaseType::Postgres),
            other => Err(CliError::ConfigParse(format!(
                "Invalid database type '{}'",
                other
            ))),
        }
    }

    /// Get a list of enabled features in order
    pub fn enabled_features(&self) -> Vec<Feature> {
        let mut features = Vec::new();

        // Base feature is always first
        if self.features.email_password {
            features.push(Feature::EmailPassword);
        }

        // Add-on features in order
        if self.features.email_verification {
            features.push(Feature::EmailVerification);
        }

        features
    }
}

/// Represents a feature that can be enabled
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Feature {
    /// Base email/password authentication
    EmailPassword,
    /// Email verification add-on
    EmailVerification,
}

impl Feature {
    /// Get the feature name for migration naming
    pub fn migration_name(&self) -> &'static str {
        match self {
            Feature::EmailPassword => "base",
            Feature::EmailVerification => "email_verification",
        }
    }

    /// Get human-readable feature name
    pub fn display_name(&self) -> &'static str {
        match self {
            Feature::EmailPassword => "Email/Password Authentication",
            Feature::EmailVerification => "Email Verification",
        }
    }

    /// Get the migration version for this feature
    pub fn version(&self) -> u32 {
        match self {
            Feature::EmailPassword => 1,
            Feature::EmailVerification => 2,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AuthKitConfig::default_config(DatabaseType::Postgres);
        assert_eq!(config.database.db_type, "postgres");
        assert!(config.features.email_password);
        assert!(!config.features.email_verification);
    }

    #[test]
    fn test_enabled_features() {
        let mut config = AuthKitConfig::default_config(DatabaseType::Postgres);
        config.features.email_verification = true;

        let features = config.enabled_features();
        assert_eq!(features.len(), 2);
        assert_eq!(features[0], Feature::EmailPassword);
        assert_eq!(features[1], Feature::EmailVerification);
    }
}
