"""
Configuration module for the test application.
Handles application settings and environment variables.
"""

import os
from dataclasses import dataclass
from typing import Optional


@dataclass
class Config:
    """Application configuration class."""

    app_name: str = "TestApp"
    version: str = "1.0.0"
    debug: bool = True
    database_url: str = "sqlite:///test.db"
    auth_secret: str = "test-secret-key-12345"
    log_level: str = "INFO"
    max_users: int = 1000
    timeout: int = 30

    def __post_init__(self):
        """Load configuration from environment variables if available."""
        self.app_name = os.getenv("APP_NAME", self.app_name)
        self.version = os.getenv("APP_VERSION", self.version)
        self.debug = os.getenv("DEBUG", "true").lower() == "true"
        self.database_url = os.getenv("DATABASE_URL", self.database_url)
        self.auth_secret = os.getenv("AUTH_SECRET", self.auth_secret)
        self.log_level = os.getenv("LOG_LEVEL", self.log_level)

        # Convert string environment variables to integers
        try:
            self.max_users = int(os.getenv("MAX_USERS", str(self.max_users)))
            self.timeout = int(os.getenv("TIMEOUT", str(self.timeout)))
        except ValueError:
            # Keep default values if conversion fails
            pass

    def get_database_config(self) -> dict:
        """Get database configuration as dictionary."""
        return {
            "url": self.database_url,
            "timeout": self.timeout,
            "debug": self.debug
        }

    def get_auth_config(self) -> dict:
        """Get authentication configuration as dictionary."""
        return {
            "secret": self.auth_secret,
            "timeout": self.timeout
        }

    def is_production(self) -> bool:
        """Check if running in production mode."""
        return not self.debug

    def __str__(self) -> str:
        """String representation of configuration."""
        return f"Config(app={self.app_name}, version={self.version}, debug={self.debug})"