"""
Authentication service for the test application.
Handles user authentication and authorization.
"""

import hashlib
import secrets
from typing import Dict, Optional, Any
from datetime import datetime, timedelta

from models.user import User


class AuthService:
    """Service class for authentication operations."""

    def __init__(self, secret_key: str = "default-secret"):
        """Initialize the authentication service."""
        self.secret_key = secret_key
        self.active_sessions: Dict[str, Dict[str, Any]] = {}
        self.failed_attempts: Dict[str, int] = {}
        self.max_failed_attempts = 3
        self.session_timeout = timedelta(hours=24)

    def _hash_password(self, password: str, salt: str = None) -> tuple[str, str]:
        """Hash a password with salt."""
        if salt is None:
            salt = secrets.token_hex(16)

        # Combine password with salt and secret key
        combined = f"{password}{salt}{self.secret_key}"
        hashed = hashlib.sha256(combined.encode()).hexdigest()

        return hashed, salt

    def _verify_password(self, password: str, hashed_password: str, salt: str) -> bool:
        """Verify a password against its hash."""
        computed_hash, _ = self._hash_password(password, salt)
        return computed_hash == hashed_password

    def authenticate(self, user: User, password: str = "testpass") -> bool:
        """
        Authenticate a user.
        For testing purposes, this uses a simple authentication mechanism.
        """
        # Check if user is locked out due to failed attempts
        if self._is_locked_out(user.email):
            return False

        # For testing, we'll use a simple authentication
        # In real applications, this would verify against stored credentials
        if user.is_active and self._verify_test_credentials(user, password):
            # Clear failed attempts on successful login
            self.failed_attempts.pop(user.email, None)
            user.update_last_login()
            return True
        else:
            # Record failed attempt
            self._record_failed_attempt(user.email)
            return False

    def _verify_test_credentials(self, user: User, password: str) -> bool:
        """Verify test credentials (simplified for testing)."""
        # For testing purposes, accept specific test passwords
        valid_passwords = ["testpass", "password123", "admin"]
        return password in valid_passwords

    def _is_locked_out(self, email: str) -> bool:
        """Check if user is locked out due to failed attempts."""
        return self.failed_attempts.get(email, 0) >= self.max_failed_attempts

    def _record_failed_attempt(self, email: str):
        """Record a failed authentication attempt."""
        self.failed_attempts[email] = self.failed_attempts.get(email, 0) + 1

    def create_session(self, user: User) -> str:
        """Create a new session for authenticated user."""
        session_id = secrets.token_urlsafe(32)
        session_data = {
            "user_id": user.user_id,
            "username": user.username,
            "email": user.email,
            "created_at": datetime.now(),
            "expires_at": datetime.now() + self.session_timeout,
            "is_active": True
        }

        self.active_sessions[session_id] = session_data
        return session_id

    def validate_session(self, session_id: str) -> Optional[Dict[str, Any]]:
        """Validate and return session data if valid."""
        if session_id not in self.active_sessions:
            return None

        session = self.active_sessions[session_id]

        # Check if session has expired
        if datetime.now() > session["expires_at"]:
            self.invalidate_session(session_id)
            return None

        # Check if session is active
        if not session["is_active"]:
            return None

        return session

    def invalidate_session(self, session_id: str) -> bool:
        """Invalidate a session."""
        if session_id in self.active_sessions:
            del self.active_sessions[session_id]
            return True
        return False

    def extend_session(self, session_id: str) -> bool:
        """Extend session expiration time."""
        session = self.validate_session(session_id)
        if session:
            session["expires_at"] = datetime.now() + self.session_timeout
            return True
        return False

    def get_active_sessions(self) -> Dict[str, Dict[str, Any]]:
        """Get all active sessions."""
        # Clean up expired sessions first
        current_time = datetime.now()
        expired_sessions = [
            session_id for session_id, session in self.active_sessions.items()
            if current_time > session["expires_at"]
        ]

        for session_id in expired_sessions:
            self.invalidate_session(session_id)

        return self.active_sessions.copy()

    def get_user_sessions(self, user_id: int) -> Dict[str, Dict[str, Any]]:
        """Get all active sessions for a specific user."""
        active_sessions = self.get_active_sessions()
        return {
            session_id: session for session_id, session in active_sessions.items()
            if session["user_id"] == user_id
        }

    def invalidate_user_sessions(self, user_id: int) -> int:
        """Invalidate all sessions for a specific user."""
        user_sessions = self.get_user_sessions(user_id)
        for session_id in user_sessions:
            self.invalidate_session(session_id)
        return len(user_sessions)

    def reset_failed_attempts(self, email: str):
        """Reset failed login attempts for a user."""
        self.failed_attempts.pop(email, None)

    def get_failed_attempts(self, email: str) -> int:
        """Get number of failed attempts for a user."""
        return self.failed_attempts.get(email, 0)

    def __str__(self) -> str:
        """String representation of the auth service."""
        active_count = len(self.get_active_sessions())
        return f"AuthService(active_sessions={active_count})"