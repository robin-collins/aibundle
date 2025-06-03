"""
Data service for the test application.
Handles data persistence and retrieval operations.
"""

import json
from typing import Dict, List, Optional, Any
from pathlib import Path

from models.user import User
from models.task import Task


class DataService:
    """Service class for data operations."""

    def __init__(self, database_url: str = "sqlite:///test.db"):
        """Initialize the data service."""
        self.database_url = database_url
        self.users: Dict[int, User] = {}
        self.tasks: Dict[int, Task] = {}
        self.data_file = Path("test_data.json")

        # Load existing data if available
        self._load_data()

    def _load_data(self):
        """Load data from file storage."""
        if self.data_file.exists():
            try:
                with open(self.data_file, 'r') as f:
                    data = json.load(f)

                # Load users
                if "users" in data:
                    for user_data in data["users"]:
                        user = User.from_dict(user_data)
                        self.users[user.user_id] = user

                # Load tasks
                if "tasks" in data:
                    for task_data in data["tasks"]:
                        task = Task.from_dict(task_data)
                        self.tasks[task.task_id] = task

            except (json.JSONDecodeError, KeyError, ValueError) as e:
                print(f"Warning: Could not load data from {self.data_file}: {e}")

    def _save_data(self):
        """Save data to file storage."""
        data = {
            "users": [user.to_dict() for user in self.users.values()],
            "tasks": [task.to_dict() for task in self.tasks.values()]
        }

        try:
            with open(self.data_file, 'w') as f:
                json.dump(data, f, indent=2)
        except Exception as e:
            print(f"Warning: Could not save data to {self.data_file}: {e}")

    # User operations
    def save_user(self, user: User) -> bool:
        """Save a user to the data store."""
        try:
            self.users[user.user_id] = user
            self._save_data()
            return True
        except Exception as e:
            print(f"Error saving user: {e}")
            return False

    def get_user(self, user_id: int) -> Optional[User]:
        """Retrieve a user by ID."""
        return self.users.get(user_id)

    def get_all_users(self) -> List[User]:
        """Get all users."""
        return list(self.users.values())

    def delete_user(self, user_id: int) -> bool:
        """Delete a user by ID."""
        if user_id in self.users:
            del self.users[user_id]
            self._save_data()
            return True
        return False

    def find_users_by_email(self, email: str) -> List[User]:
        """Find users by email address."""
        return [user for user in self.users.values() if user.email == email]

    def get_active_users(self) -> List[User]:
        """Get all active users."""
        return [user for user in self.users.values() if user.is_active]

    # Task operations
    def save_task(self, task: Task) -> bool:
        """Save a task to the data store."""
        try:
            self.tasks[task.task_id] = task
            self._save_data()
            return True
        except Exception as e:
            print(f"Error saving task: {e}")
            return False

    def get_task(self, task_id: int) -> Optional[Task]:
        """Retrieve a task by ID."""
        return self.tasks.get(task_id)

    def get_all_tasks(self) -> List[Task]:
        """Get all tasks."""
        return list(self.tasks.values())

    def delete_task(self, task_id: int) -> bool:
        """Delete a task by ID."""
        if task_id in self.tasks:
            del self.tasks[task_id]
            self._save_data()
            return True
        return False

    def get_tasks_by_user(self, user_id: int) -> List[Task]:
        """Get all tasks assigned to a specific user."""
        return [task for task in self.tasks.values() if task.assigned_to == user_id]

    def get_tasks_by_status(self, status: str) -> List[Task]:
        """Get all tasks with a specific status."""
        return [task for task in self.tasks.values() if task.status == status]

    def get_tasks_by_priority(self, priority: str) -> List[Task]:
        """Get all tasks with a specific priority."""
        return [task for task in self.tasks.values() if task.priority == priority]

    def get_overdue_tasks(self) -> List[Task]:
        """Get all overdue tasks."""
        return [task for task in self.tasks.values() if task.is_overdue()]

    # Statistics
    def get_user_count(self) -> int:
        """Get total number of users."""
        return len(self.users)

    def get_task_count(self) -> int:
        """Get total number of tasks."""
        return len(self.tasks)

    def get_active_user_count(self) -> int:
        """Get number of active users."""
        return len(self.get_active_users())

    def get_task_statistics(self) -> Dict[str, int]:
        """Get task statistics by status."""
        stats = {}
        for task in self.tasks.values():
            status = task.status
            stats[status] = stats.get(status, 0) + 1
        return stats

    def __str__(self) -> str:
        """String representation of the data service."""
        return f"DataService(users={len(self.users)}, tasks={len(self.tasks)})"