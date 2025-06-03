"""
User model for the test application.
"""

from dataclasses import dataclass, field
from datetime import datetime
from typing import Optional, Dict, Any


@dataclass
class User:
    """User data model."""

    user_id: int
    username: str
    email: str
    is_active: bool = True
    created_at: datetime = field(default_factory=datetime.now)
    last_login: Optional[datetime] = None
    profile_data: Dict[str, Any] = field(default_factory=dict)

    def __post_init__(self):
        """Validation and post-initialization logic."""
        if not self.username:
            raise ValueError("Username cannot be empty")
        if not self.email or "@" not in self.email:
            raise ValueError("Invalid email address")
        if self.user_id < 0:
            raise ValueError("User ID must be positive")

    def to_dict(self) -> Dict[str, Any]:
        """Convert user to dictionary representation."""
        return {
            "user_id": self.user_id,
            "username": self.username,
            "email": self.email,
            "is_active": self.is_active,
            "created_at": self.created_at.isoformat() if self.created_at else None,
            "last_login": self.last_login.isoformat() if self.last_login else None,
            "profile_data": self.profile_data
        }

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "User":
        """Create User instance from dictionary."""
        # Parse datetime strings back to datetime objects
        if "created_at" in data and data["created_at"]:
            data["created_at"] = datetime.fromisoformat(data["created_at"])
        if "last_login" in data and data["last_login"]:
            data["last_login"] = datetime.fromisoformat(data["last_login"])

        return cls(**data)

    def update_last_login(self):
        """Update the last login timestamp."""
        self.last_login = datetime.now()

    def deactivate(self):
        """Deactivate the user account."""
        self.is_active = False

    def activate(self):
        """Activate the user account."""
        self.is_active = True

    def update_profile(self, **kwargs):
        """Update profile data."""
        self.profile_data.update(kwargs)

    def __str__(self) -> str:
        """String representation of the user."""
        status = "active" if self.is_active else "inactive"
        return f"User({self.username}, {self.email}, {status})"