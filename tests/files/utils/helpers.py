"""
Helper utilities for the test application.
"""

import re
import json
from typing import Any, Dict, List, Optional, Union
from datetime import datetime
from pathlib import Path


def validate_input(input_value: Any, input_type: str = "email") -> bool:
    """
    Validate various types of input.

    Args:
        input_value: The value to validate
        input_type: Type of validation to perform

    Returns:
        True if valid, False otherwise
    """
    if input_type == "email":
        return validate_email(input_value)
    elif input_type == "username":
        return validate_username(input_value)
    elif input_type == "password":
        return validate_password(input_value)
    elif input_type == "phone":
        return validate_phone(input_value)
    else:
        return False


def validate_email(email: str) -> bool:
    """Validate email address format."""
    if not isinstance(email, str):
        return False

    # Basic email regex pattern
    pattern = r'^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$'
    return re.match(pattern, email) is not None


def validate_username(username: str) -> bool:
    """Validate username format."""
    if not isinstance(username, str):
        return False

    # Username should be 3-20 characters, alphanumeric and underscores only
    pattern = r'^[a-zA-Z0-9_]{3,20}$'
    return re.match(pattern, username) is not None


def validate_password(password: str) -> bool:
    """Validate password strength."""
    if not isinstance(password, str):
        return False

    # Password should be at least 8 characters with at least one number
    if len(password) < 8:
        return False

    has_number = re.search(r'\d', password) is not None
    has_letter = re.search(r'[a-zA-Z]', password) is not None

    return has_number and has_letter


def validate_phone(phone: str) -> bool:
    """Validate phone number format."""
    if not isinstance(phone, str):
        return False

    # Remove common separators
    cleaned_phone = re.sub(r'[\s\-\(\)\.]+', '', phone)

    # Check if it's all digits and has reasonable length
    return cleaned_phone.isdigit() and 10 <= len(cleaned_phone) <= 15


def format_output(title: str, data: Union[Dict, List, str], indent: int = 2) -> str:
    """
    Format data for output display.

    Args:
        title: Title for the output section
        data: Data to format
        indent: Indentation level

    Returns:
        Formatted string representation
    """
    separator = "=" * 50

    if isinstance(data, (dict, list)):
        formatted_data = json.dumps(data, indent=indent, default=str)
    else:
        formatted_data = str(data)

    return f"\n{separator}\n{title}\n{separator}\n{formatted_data}\n{separator}\n"


def sanitize_filename(filename: str) -> str:
    """Sanitize filename by removing invalid characters."""
    # Replace invalid characters with underscores
    sanitized = re.sub(r'[<>:"/\\|?*]', '_', filename)

    # Remove leading/trailing spaces and dots
    sanitized = sanitized.strip(' .')

    # Limit length
    if len(sanitized) > 255:
        sanitized = sanitized[:255]

    return sanitized


def deep_merge_dicts(dict1: Dict, dict2: Dict) -> Dict:
    """
    Deep merge two dictionaries.

    Args:
        dict1: First dictionary
        dict2: Second dictionary (takes precedence)

    Returns:
        Merged dictionary
    """
    result = dict1.copy()

    for key, value in dict2.items():
        if key in result and isinstance(result[key], dict) and isinstance(value, dict):
            result[key] = deep_merge_dicts(result[key], value)
        else:
            result[key] = value

    return result


def flatten_dict(d: Dict, parent_key: str = '', sep: str = '.') -> Dict:
    """
    Flatten a nested dictionary.

    Args:
        d: Dictionary to flatten
        parent_key: Parent key prefix
        sep: Separator for nested keys

    Returns:
        Flattened dictionary
    """
    items = []

    for k, v in d.items():
        new_key = f"{parent_key}{sep}{k}" if parent_key else k

        if isinstance(v, dict):
            items.extend(flatten_dict(v, new_key, sep=sep).items())
        else:
            items.append((new_key, v))

    return dict(items)


def chunk_list(lst: List, chunk_size: int) -> List[List]:
    """
    Split a list into chunks of specified size.

    Args:
        lst: List to chunk
        chunk_size: Size of each chunk

    Returns:
        List of chunks
    """
    return [lst[i:i + chunk_size] for i in range(0, len(lst), chunk_size)]


def safe_get(dictionary: Dict, key_path: str, default: Any = None, sep: str = '.') -> Any:
    """
    Safely get a value from a nested dictionary using dot notation.

    Args:
        dictionary: Dictionary to search
        key_path: Dot-separated key path (e.g., 'user.profile.name')
        default: Default value if key not found
        sep: Separator for key path

    Returns:
        Value if found, default otherwise
    """
    keys = key_path.split(sep)
    current = dictionary

    try:
        for key in keys:
            current = current[key]
        return current
    except (KeyError, TypeError):
        return default


def timestamp_to_string(timestamp: datetime, format_str: str = "%Y-%m-%d %H:%M:%S") -> str:
    """Convert datetime to formatted string."""
    return timestamp.strftime(format_str)


def string_to_timestamp(date_string: str, format_str: str = "%Y-%m-%d %H:%M:%S") -> datetime:
    """Convert formatted string to datetime."""
    return datetime.strptime(date_string, format_str)


def ensure_directory(path: Union[str, Path]) -> Path:
    """Ensure directory exists, create if necessary."""
    path_obj = Path(path)
    path_obj.mkdir(parents=True, exist_ok=True)
    return path_obj


def file_size_human_readable(size_bytes: int) -> str:
    """Convert file size in bytes to human readable format."""
    if size_bytes == 0:
        return "0 B"

    size_names = ["B", "KB", "MB", "GB", "TB"]
    i = 0
    size = float(size_bytes)

    while size >= 1024.0 and i < len(size_names) - 1:
        size /= 1024.0
        i += 1

    return f"{size:.2f} {size_names[i]}"


def retry_operation(func, max_attempts: int = 3, delay: float = 1.0):
    """
    Retry an operation with exponential backoff.

    Args:
        func: Function to retry
        max_attempts: Maximum number of attempts
        delay: Initial delay between attempts

    Returns:
        Function result if successful

    Raises:
        Last exception if all attempts fail
    """
    import time

    last_exception = None

    for attempt in range(max_attempts):
        try:
            return func()
        except Exception as e:
            last_exception = e
            if attempt < max_attempts - 1:
                time.sleep(delay * (2 ** attempt))  # Exponential backoff

    raise last_exception