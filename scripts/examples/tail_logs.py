"""
@name: Tail Logs
@description: Quick log tailing panel with preset log paths
@author: NeonShell Team
@version: 1.0.0
"""

from neonshell import (
    register_command,
    run_command,
    show_quick_pick,
    show_notification,
    log,
    get_active_session,
)

# Common log paths for different systems
LOG_PRESETS = {
    "System Log (syslog)": "/var/log/syslog",
    "Auth Log": "/var/log/auth.log",
    "Kernel Log": "/var/log/kern.log",
    "Nginx Access": "/var/log/nginx/access.log",
    "Nginx Error": "/var/log/nginx/error.log",
    "Apache Access": "/var/log/apache2/access.log",
    "Apache Error": "/var/log/apache2/error.log",
    "Docker Logs": "/var/lib/docker/containers/*/*.log",
    "PostgreSQL": "/var/log/postgresql/postgresql-*-main.log",
    "MySQL": "/var/log/mysql/error.log",
    "Redis": "/var/log/redis/redis-server.log",
    "Custom...": None,
}


@register_command("tail-logs.quick-tail")
def quick_tail():
    """Show a quick picker to select and tail a log file."""
    session = get_active_session()
    if not session:
        show_notification("No active session", type="warning")
        return
    
    # Show log file picker
    options = list(LOG_PRESETS.keys())
    selected = show_quick_pick(options, placeholder="Select a log file to tail...")
    
    if not selected:
        return
    
    log_path = LOG_PRESETS.get(selected)
    
    if selected == "Custom...":
        log_path = show_input(
            placeholder="Enter the log file path...",
            prompt="Log File Path",
        )
        if not log_path:
            return
    
    # Build the tail command
    lines = 100  # Default number of lines
    command = f"sudo tail -f -n {lines} {log_path}"
    
    log(f"Tail Logs: Tailing {log_path}")
    run_command(session['id'], command)
    
    show_notification(f"Tailing: {log_path}", type="info")


@register_command("tail-logs.tail-with-filter")
def tail_with_filter():
    """Tail a log file with grep filter."""
    session = get_active_session()
    if not session:
        show_notification("No active session", type="warning")
        return
    
    # Get log path
    options = list(LOG_PRESETS.keys())
    selected = show_quick_pick(options, placeholder="Select a log file...")
    
    if not selected:
        return
    
    log_path = LOG_PRESETS.get(selected)
    if selected == "Custom..." or not log_path:
        log_path = show_input(placeholder="Enter log path...")
        if not log_path:
            return
    
    # Get filter pattern
    pattern = show_input(
        placeholder="Enter grep pattern (e.g., 'error', 'WARNING')...",
        prompt="Filter Pattern",
    )
    
    if not pattern:
        pattern = ""
    
    # Build command with optional filter
    if pattern:
        command = f"sudo tail -f -n 100 {log_path} | grep --line-buffered '{pattern}'"
    else:
        command = f"sudo tail -f -n 100 {log_path}"
    
    log(f"Tail Logs: Tailing {log_path} with filter '{pattern}'")
    run_command(session['id'], command)


@register_command("tail-logs.journalctl")
def journalctl_follow():
    """Follow systemd journal logs."""
    session = get_active_session()
    if not session:
        show_notification("No active session", type="warning")
        return
    
    options = [
        "All logs",
        "Kernel logs",
        "Current boot",
        "Priority: Error and above",
        "Priority: Warning and above",
        "Specific unit...",
    ]
    
    selected = show_quick_pick(options, placeholder="Select journal view...")
    
    if not selected:
        return
    
    commands = {
        "All logs": "sudo journalctl -f",
        "Kernel logs": "sudo journalctl -kf",
        "Current boot": "sudo journalctl -b -f",
        "Priority: Error and above": "sudo journalctl -f -p err",
        "Priority: Warning and above": "sudo journalctl -f -p warning",
    }
    
    if selected == "Specific unit...":
        unit = show_input(placeholder="Enter unit name (e.g., nginx, docker)...")
        if not unit:
            return
        command = f"sudo journalctl -u {unit} -f"
    else:
        command = commands.get(selected, "sudo journalctl -f")
    
    log(f"Tail Logs: Running {command}")
    run_command(session['id'], command)


# Helper function for input (would be provided by neonshell module)
def show_input(placeholder="", prompt="Input"):
    """Show an input dialog. This would be provided by the neonshell module."""
    # Placeholder implementation - real implementation uses IPC to frontend
    return None




