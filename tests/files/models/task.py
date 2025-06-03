"""
Task model for the test application.
"""

from dataclasses import dataclass, field
from datetime import datetime
from typing import Optional, Dict, Any, List
from enum import Enum


class TaskStatus(Enum):
    """Task status enumeration."""
    PENDING = "pending"
    IN_PROGRESS = "in_progress"
    COMPLETED = "completed"
    CANCELLED = "cancelled"


class TaskPriority(Enum):
    """Task priority enumeration."""
    LOW = "low"
    MEDIUM = "medium"
    HIGH = "high"
    URGENT = "urgent"


@dataclass
class Task:
    """Task data model."""

    task_id: int
    title: str
    description: str
    assigned_to: int
    priority: str = "medium"
    status: str = "pending"
    created_at: datetime = field(default_factory=datetime.now)
    updated_at: datetime = field(default_factory=datetime.now)
    due_date: Optional[datetime] = None
    completed_at: Optional[datetime] = None
    tags: List[str] = field(default_factory=list)
    metadata: Dict[str, Any] = field(default_factory=dict)

    def __post_init__(self):
        """Validation and post-initialization logic."""
        if not self.title:
            raise ValueError("Title cannot be empty")
        if not self.description:
            raise ValueError("Description cannot be empty")
        if self.task_id < 0:
            raise ValueError("Task ID must be positive")
        if self.assigned_to < 0:
            raise ValueError("Assigned user ID must be positive")

        # Validate priority
        valid_priorities = [p.value for p in TaskPriority]
        if self.priority not in valid_priorities:
            raise ValueError(f"Priority must be one of: {valid_priorities}")

        # Validate status
        valid_statuses = [s.value for s in TaskStatus]
        if self.status not in valid_statuses:
            raise ValueError(f"Status must be one of: {valid_statuses}")

    def to_dict(self) -> Dict[str, Any]:
        """Convert task to dictionary representation."""
        return {
            "task_id": self.task_id,
            "title": self.title,
            "description": self.description,
            "assigned_to": self.assigned_to,
            "priority": self.priority,
            "status": self.status,
            "created_at": self.created_at.isoformat() if self.created_at else None,
            "updated_at": self.updated_at.isoformat() if self.updated_at else None,
            "due_date": self.due_date.isoformat() if self.due_date else None,
            "completed_at": self.completed_at.isoformat() if self.completed_at else None,
            "tags": self.tags,
            "metadata": self.metadata
        }

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "Task":
        """Create Task instance from dictionary."""
        # Parse datetime strings back to datetime objects
        datetime_fields = ["created_at", "updated_at", "due_date", "completed_at"]
        for field_name in datetime_fields:
            if field_name in data and data[field_name]:
                data[field_name] = datetime.fromisoformat(data[field_name])

        return cls(**data)

    def update_status(self, new_status: str):
        """Update task status."""
        valid_statuses = [s.value for s in TaskStatus]
        if new_status not in valid_statuses:
            raise ValueError(f"Status must be one of: {valid_statuses}")

        self.status = new_status
        self.updated_at = datetime.now()

        # Set completion timestamp if completed
        if new_status == TaskStatus.COMPLETED.value:
            self.completed_at = datetime.now()

    def update_priority(self, new_priority: str):
        """Update task priority."""
        valid_priorities = [p.value for p in TaskPriority]
        if new_priority not in valid_priorities:
            raise ValueError(f"Priority must be one of: {valid_priorities}")

        self.priority = new_priority
        self.updated_at = datetime.now()

    def add_tag(self, tag: str):
        """Add a tag to the task."""
        if tag not in self.tags:
            self.tags.append(tag)
            self.updated_at = datetime.now()

    def remove_tag(self, tag: str):
        """Remove a tag from the task."""
        if tag in self.tags:
            self.tags.remove(tag)
            self.updated_at = datetime.now()

    def set_due_date(self, due_date: datetime):
        """Set task due date."""
        self.due_date = due_date
        self.updated_at = datetime.now()

    def is_overdue(self) -> bool:
        """Check if task is overdue."""
        if not self.due_date:
            return False
        return self.due_date < datetime.now() and self.status != TaskStatus.COMPLETED.value

    def __str__(self) -> str:
        """String representation of the task."""
        return f"Task({self.task_id}: {self.title}, {self.status}, {self.priority})"