"""
Logging utilities for the test application.
"""

import logging
import sys
from pathlib import Path
from datetime import datetime
from typing import Optional


def setup_logger(
    name: str = "TestApp",
    level: str = "INFO",
    log_file: Optional[str] = None,
    format_string: Optional[str] = None
) -> logging.Logger:
    """
    Set up and configure application logger.

    Args:
        name: Logger name
        level: Logging level (DEBUG, INFO, WARNING, ERROR, CRITICAL)
        log_file: Optional file path for logging to file
        format_string: Custom format string for log messages

    Returns:
        Configured logger instance
    """
    # Create logger
    logger = logging.getLogger(name)

    # Clear any existing handlers to avoid duplicates
    logger.handlers.clear()

    # Set logging level
    level_map = {
        "DEBUG": logging.DEBUG,
        "INFO": logging.INFO,
        "WARNING": logging.WARNING,
        "ERROR": logging.ERROR,
        "CRITICAL": logging.CRITICAL
    }
    logger.setLevel(level_map.get(level.upper(), logging.INFO))

    # Create formatter
    if format_string is None:
        format_string = "%(asctime)s - %(name)s - %(levelname)s - %(message)s"

    formatter = logging.Formatter(format_string)

    # Console handler
    console_handler = logging.StreamHandler(sys.stdout)
    console_handler.setLevel(logger.level)
    console_handler.setFormatter(formatter)
    logger.addHandler(console_handler)

    # File handler (optional)
    if log_file:
        # Create log directory if it doesn't exist
        log_path = Path(log_file)
        log_path.parent.mkdir(parents=True, exist_ok=True)

        file_handler = logging.FileHandler(log_file)
        file_handler.setLevel(logger.level)
        file_handler.setFormatter(formatter)
        logger.addHandler(file_handler)

    # Prevent duplicate logs by not propagating to root logger
    logger.propagate = False

    return logger


def get_logger(name: str = "TestApp") -> logging.Logger:
    """Get an existing logger by name."""
    return logging.getLogger(name)


def create_rotating_file_logger(
    name: str = "TestApp",
    log_file: str = "app.log",
    max_bytes: int = 10485760,  # 10MB
    backup_count: int = 5,
    level: str = "INFO"
) -> logging.Logger:
    """
    Create a logger with rotating file handler.

    Args:
        name: Logger name
        log_file: Log file path
        max_bytes: Maximum file size before rotation
        backup_count: Number of backup files to keep
        level: Logging level

    Returns:
        Configured logger with rotating file handler
    """
    from logging.handlers import RotatingFileHandler

    logger = logging.getLogger(name)
    logger.handlers.clear()

    # Set level
    level_map = {
        "DEBUG": logging.DEBUG,
        "INFO": logging.INFO,
        "WARNING": logging.WARNING,
        "ERROR": logging.ERROR,
        "CRITICAL": logging.CRITICAL
    }
    logger.setLevel(level_map.get(level.upper(), logging.INFO))

    # Create formatter
    formatter = logging.Formatter(
        "%(asctime)s - %(name)s - %(levelname)s - %(funcName)s:%(lineno)d - %(message)s"
    )

    # Create log directory if it doesn't exist
    log_path = Path(log_file)
    log_path.parent.mkdir(parents=True, exist_ok=True)

    # Rotating file handler
    file_handler = RotatingFileHandler(
        log_file, maxBytes=max_bytes, backupCount=backup_count
    )
    file_handler.setLevel(logger.level)
    file_handler.setFormatter(formatter)
    logger.addHandler(file_handler)

    # Console handler
    console_handler = logging.StreamHandler(sys.stdout)
    console_handler.setLevel(logging.WARNING)  # Only warnings and above to console
    console_handler.setFormatter(formatter)
    logger.addHandler(console_handler)

    logger.propagate = False

    return logger


def log_function_entry_exit(func):
    """
    Decorator to log function entry and exit.
    """
    def wrapper(*args, **kwargs):
        logger = get_logger()
        func_name = func.__name__

        # Log entry
        logger.debug(f"Entering function: {func_name}")

        try:
            # Execute function
            result = func(*args, **kwargs)

            # Log successful exit
            logger.debug(f"Exiting function: {func_name} (success)")
            return result

        except Exception as e:
            # Log exception
            logger.error(f"Exception in function {func_name}: {e}")
            raise

    return wrapper


def log_execution_time(func):
    """
    Decorator to log function execution time.
    """
    def wrapper(*args, **kwargs):
        import time

        logger = get_logger()
        func_name = func.__name__

        start_time = time.time()

        try:
            result = func(*args, **kwargs)
            execution_time = time.time() - start_time
            logger.info(f"Function {func_name} executed in {execution_time:.4f} seconds")
            return result

        except Exception as e:
            execution_time = time.time() - start_time
            logger.error(f"Function {func_name} failed after {execution_time:.4f} seconds: {e}")
            raise

    return wrapper