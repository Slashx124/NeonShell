"""
NeonShell Python Scripting API

This module provides the API for writing NeonShell automation scripts.
Scripts can register hooks, commands, and interact with SSH sessions.

Example usage:
    from neonshell import hook, run_command, log

    @hook("on_connect")
    def my_hook(session):
        log(f"Connected to {session['host']}")
        run_command(session['id'], "echo Hello!")
"""

from typing import Callable, Dict, Any, List, Optional
import json
import sys

# Version of the scripting API
__version__ = "1.0.0"

# Internal storage for registered hooks and commands
_hooks: Dict[str, List[Callable]] = {}
_commands: Dict[str, Dict[str, Any]] = {}
_config: Dict[str, Any] = {}
_session_vars: Dict[str, Dict[str, Any]] = {}


def hook(event: str):
    """
    Decorator to register a function as an event hook.
    
    Supported events:
        - on_connect: Called when a session connects
        - on_disconnect: Called when a session disconnects
        - on_data: Called when data is received (can modify output)
        - on_command: Called before a command is sent
        - on_error: Called when an error occurs
    
    Example:
        @hook("on_connect")
        def my_connect_handler(session):
            print(f"Connected to {session['host']}")
    """
    def decorator(func: Callable):
        if event not in _hooks:
            _hooks[event] = []
        _hooks[event].append(func)
        return func
    return decorator


def register_command(command_id: str, description: str = ""):
    """
    Decorator to register a function as a command.
    
    Commands appear in the command palette and can be bound to hotkeys.
    
    Example:
        @register_command("my-script.hello", "Say hello")
        def hello_command():
            show_notification("Hello!")
    """
    def decorator(func: Callable):
        _commands[command_id] = {
            "id": command_id,
            "name": func.__name__.replace("_", " ").title(),
            "description": description or func.__doc__ or "",
            "handler": func,
        }
        return func
    return decorator


def run_command(session_id: str, command: str) -> None:
    """
    Send a command to the terminal.
    
    Args:
        session_id: The session to send the command to
        command: The command string to execute
    """
    _ipc_call("run_command", {"session_id": session_id, "command": command})


def run_command_capture(session_id: str, command: str, timeout: int = 30) -> Dict[str, Any]:
    """
    Run a command and capture its output.
    
    Args:
        session_id: The session to run the command on
        command: The command to execute
        timeout: Maximum time to wait in seconds
    
    Returns:
        Dict with 'success', 'output', and optionally 'error' keys
    """
    return _ipc_call("run_command_capture", {
        "session_id": session_id,
        "command": command,
        "timeout": timeout,
    })


def get_active_session() -> Optional[Dict[str, Any]]:
    """
    Get the currently active session.
    
    Returns:
        Session dict with id, host, port, username, state, etc.
        None if no active session.
    """
    return _ipc_call("get_active_session", {})


def log(message: str, level: str = "info") -> None:
    """
    Write a message to the script log.
    
    Args:
        message: The message to log
        level: Log level (debug, info, warn, error)
    """
    _ipc_call("log", {"message": message, "level": level})


def show_notification(message: str, type: str = "info", title: str = None) -> None:
    """
    Show a notification to the user.
    
    Args:
        message: The notification message
        type: Notification type (info, success, warning, error)
        title: Optional title
    """
    _ipc_call("show_notification", {
        "message": message,
        "type": type,
        "title": title,
    })


def show_quick_pick(
    options: List[str],
    placeholder: str = "",
    can_pick_many: bool = False
) -> Optional[str]:
    """
    Show a quick pick dialog.
    
    Args:
        options: List of options to choose from
        placeholder: Placeholder text
        can_pick_many: Allow multiple selections
    
    Returns:
        Selected option(s) or None if cancelled
    """
    return _ipc_call("show_quick_pick", {
        "options": options,
        "placeholder": placeholder,
        "can_pick_many": can_pick_many,
    })


def show_input(
    placeholder: str = "",
    prompt: str = "",
    password: bool = False,
    value: str = ""
) -> Optional[str]:
    """
    Show an input dialog.
    
    Args:
        placeholder: Placeholder text
        prompt: Input prompt label
        password: Mask input as password
        value: Default value
    
    Returns:
        Entered text or None if cancelled
    """
    return _ipc_call("show_input", {
        "placeholder": placeholder,
        "prompt": prompt,
        "password": password,
        "value": value,
    })


def show_panel(widget: Dict[str, Any]) -> None:
    """
    Show a custom panel.
    
    Args:
        widget: Widget definition dict
    """
    _ipc_call("show_panel", {"widget": widget})


def create_widget(definition: Dict[str, Any]) -> Dict[str, Any]:
    """
    Create a UI widget definition.
    
    Args:
        definition: Widget definition with type, title, content, etc.
    
    Returns:
        Widget definition dict
    """
    return definition


def get_config(key: str, default: Any = None) -> Any:
    """
    Get a script configuration value.
    
    Args:
        key: Configuration key
        default: Default value if not set
    
    Returns:
        Configuration value
    """
    return _config.get(key, default)


def set_config(key: str, value: Any) -> None:
    """
    Set a script configuration value.
    
    Args:
        key: Configuration key
        value: Value to set
    """
    _config[key] = value
    _ipc_call("set_config", {"key": key, "value": value})


def get_session_var(session_id: str, key: str, default: Any = None) -> Any:
    """
    Get a session-specific variable.
    
    Args:
        session_id: Session ID
        key: Variable key
        default: Default value
    
    Returns:
        Variable value
    """
    if session_id not in _session_vars:
        return default
    return _session_vars[session_id].get(key, default)


def set_session_var(session_id: str, key: str, value: Any) -> None:
    """
    Set a session-specific variable.
    
    Args:
        session_id: Session ID
        key: Variable key
        value: Value to set
    """
    if session_id not in _session_vars:
        _session_vars[session_id] = {}
    _session_vars[session_id][key] = value


def create_port_forward(
    session_id: str,
    forward_type: str,
    local_port: int,
    remote_host: str,
    remote_port: int
) -> Dict[str, Any]:
    """
    Create a port forward.
    
    Args:
        session_id: Session ID
        forward_type: Type (local, remote, dynamic)
        local_port: Local port
        remote_host: Remote host
        remote_port: Remote port
    
    Returns:
        Dict with success status and forward ID
    """
    return _ipc_call("create_port_forward", {
        "session_id": session_id,
        "forward_type": forward_type,
        "local_port": local_port,
        "remote_host": remote_host,
        "remote_port": remote_port,
    })


def remove_port_forward(session_id: str, forward_id: str) -> None:
    """Remove a port forward."""
    _ipc_call("remove_port_forward", {
        "session_id": session_id,
        "forward_id": forward_id,
    })


def list_port_forwards(session_id: str) -> List[Dict[str, Any]]:
    """List all port forwards for a session."""
    return _ipc_call("list_port_forwards", {"session_id": session_id}) or []


def test_connection(host: str, port: int = 22, timeout: int = 5) -> Dict[str, Any]:
    """
    Test if a host is reachable.
    
    Args:
        host: Host to test
        port: Port to test
        timeout: Connection timeout
    
    Returns:
        Dict with success status
    """
    return _ipc_call("test_connection", {
        "host": host,
        "port": port,
        "timeout": timeout,
    })


def modify_connection(session_id: str, changes: Dict[str, Any]) -> None:
    """
    Modify connection parameters.
    
    Args:
        session_id: Session ID
        changes: Dict of parameters to change
    """
    _ipc_call("modify_connection", {
        "session_id": session_id,
        "changes": changes,
    })


def modify_output(session_id: str, data: str) -> str:
    """
    Modify terminal output (for use in on_data hook).
    
    Args:
        session_id: Session ID
        data: Modified output data
    
    Returns:
        The modified data
    """
    return data


def _ipc_call(method: str, params: Dict[str, Any]) -> Any:
    """
    Internal: Make an IPC call to the NeonShell backend.
    
    In the actual implementation, this communicates with
    the Tauri backend via stdin/stdout JSON-RPC.
    """
    # Placeholder implementation
    # Real implementation uses subprocess IPC
    request = {
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": 1,
    }
    
    # In production, this would write to stdout and read from stdin
    # For now, return mock responses for development
    if method == "get_active_session":
        return None
    elif method == "show_quick_pick":
        return None
    elif method == "show_input":
        return None
    elif method in ("log", "show_notification", "run_command", "set_config"):
        return None
    elif method == "run_command_capture":
        return {"success": False, "output": "", "error": "Not connected"}
    elif method == "list_port_forwards":
        return []
    elif method == "test_connection":
        return {"success": False}
    elif method == "create_port_forward":
        return {"success": False, "error": "Not implemented"}
    
    return None


# Export all public functions
__all__ = [
    "hook",
    "register_command",
    "run_command",
    "run_command_capture",
    "get_active_session",
    "log",
    "show_notification",
    "show_quick_pick",
    "show_input",
    "show_panel",
    "create_widget",
    "get_config",
    "set_config",
    "get_session_var",
    "set_session_var",
    "create_port_forward",
    "remove_port_forward",
    "list_port_forwards",
    "test_connection",
    "modify_connection",
    "modify_output",
]

