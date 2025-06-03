#!/usr/bin/env python3
"""
Main entry point for the test application.
This file demonstrates a typical Python application structure.
"""

import sys
import os
from pathlib import Path

# Add the current directory to Python path for imports
sys.path.insert(0, str(Path(__file__).parent))

from config import Config
from models.user import User
from models.task import Task
from services.data_service import DataService
from services.auth_service import AuthService
from utils.logger import setup_logger
from utils.helpers import validate_input, format_output


def main():
    """Main application function."""
    # Setup logging
    logger = setup_logger()
    logger.info("Starting application...")

    # Load configuration
    config = Config()
    logger.info(f"Loaded configuration: {config.app_name} v{config.version}")

    # Initialize services
    data_service = DataService(config.database_url)
    auth_service = AuthService(config.auth_secret)

    # Create some test data
    user = User(
        user_id=1,
        username="testuser",
        email="test@example.com",
        is_active=True
    )

    task = Task(
        task_id=101,
        title="Sample Task",
        description="This is a sample task for testing",
        assigned_to=user.user_id,
        priority="high"
    )

    # Validate and process data
    if validate_input(user.email):
        logger.info(f"Valid user email: {user.email}")

        # Authenticate user
        if auth_service.authenticate(user):
            logger.info(f"User {user.username} authenticated successfully")

            # Save data
            data_service.save_user(user)
            data_service.save_task(task)

            # Get and display results
            saved_user = data_service.get_user(user.user_id)
            saved_task = data_service.get_task(task.task_id)

            print(format_output("User Data", saved_user.to_dict()))
            print(format_output("Task Data", saved_task.to_dict()))

            logger.info("Application completed successfully")
        else:
            logger.error("Authentication failed")
            return 1
    else:
        logger.error("Invalid user email")
        return 1

    return 0


if __name__ == "__main__":
    exit_code = main()
    sys.exit(exit_code)