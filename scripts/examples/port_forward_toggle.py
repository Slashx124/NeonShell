"""
@name: Port Forward Toggle
@description: Easy management of SSH port forwards with quick toggles
@author: NeonShell Team
@version: 1.0.0
"""

from neonshell import (
    register_command,
    show_quick_pick,
    show_notification,
    show_input,
    log,
    get_active_session,
    create_port_forward,
    remove_port_forward,
    list_port_forwards,
    get_session_var,
    set_session_var,
)


# Preset port forward configurations
FORWARD_PRESETS = {
    "MySQL Local": {"local_port": 3306, "remote_host": "localhost", "remote_port": 3306},
    "PostgreSQL Local": {"local_port": 5432, "remote_host": "localhost", "remote_port": 5432},
    "Redis Local": {"local_port": 6379, "remote_host": "localhost", "remote_port": 6379},
    "MongoDB Local": {"local_port": 27017, "remote_host": "localhost", "remote_port": 27017},
    "Web Server (80)": {"local_port": 8080, "remote_host": "localhost", "remote_port": 80},
    "HTTPS (443)": {"local_port": 8443, "remote_host": "localhost", "remote_port": 443},
    "Grafana": {"local_port": 3000, "remote_host": "localhost", "remote_port": 3000},
    "Prometheus": {"local_port": 9090, "remote_host": "localhost", "remote_port": 9090},
    "Jupyter Notebook": {"local_port": 8888, "remote_host": "localhost", "remote_port": 8888},
}


@register_command("port-forward.quick-forward")
def quick_forward():
    """Set up a port forward using presets."""
    session = get_active_session()
    if not session:
        show_notification("No active session", type="warning")
        return
    
    # Build options list with current status
    active_forwards = list_port_forwards(session['id'])
    active_local_ports = {f['local_port'] for f in active_forwards}
    
    options = []
    for name, config in FORWARD_PRESETS.items():
        status = "✓ Active" if config['local_port'] in active_local_ports else ""
        options.append(f"{name} (:{config['local_port']}) {status}".strip())
    
    options.append("Custom...")
    
    selected = show_quick_pick(options, placeholder="Select port forward...")
    
    if not selected:
        return
    
    if selected.startswith("Custom"):
        custom_forward()
        return
    
    # Extract preset name (before the port number)
    preset_name = selected.split(" (")[0]
    config = FORWARD_PRESETS.get(preset_name)
    
    if not config:
        show_notification("Invalid preset", type="error")
        return
    
    # Toggle the forward
    if config['local_port'] in active_local_ports:
        # Remove existing forward
        for f in active_forwards:
            if f['local_port'] == config['local_port']:
                remove_port_forward(session['id'], f['id'])
                show_notification(f"Removed: localhost:{config['local_port']}", type="info")
                break
    else:
        # Create new forward
        result = create_port_forward(
            session['id'],
            forward_type="local",
            local_port=config['local_port'],
            remote_host=config['remote_host'],
            remote_port=config['remote_port'],
        )
        
        if result['success']:
            show_notification(
                f"Forwarding: localhost:{config['local_port']} → {config['remote_host']}:{config['remote_port']}",
                type="success"
            )
        else:
            show_notification(f"Failed: {result.get('error', 'Unknown error')}", type="error")


@register_command("port-forward.custom")
def custom_forward():
    """Set up a custom port forward."""
    session = get_active_session()
    if not session:
        show_notification("No active session", type="warning")
        return
    
    # Get local port
    local_port_str = show_input(
        placeholder="Enter local port (e.g., 8080)...",
        prompt="Local Port",
    )
    
    if not local_port_str:
        return
    
    try:
        local_port = int(local_port_str)
    except ValueError:
        show_notification("Invalid port number", type="error")
        return
    
    # Get remote host
    remote_host = show_input(
        placeholder="Enter remote host (default: localhost)...",
        prompt="Remote Host",
    ) or "localhost"
    
    # Get remote port
    remote_port_str = show_input(
        placeholder=f"Enter remote port (default: {local_port})...",
        prompt="Remote Port",
    ) or str(local_port)
    
    try:
        remote_port = int(remote_port_str)
    except ValueError:
        show_notification("Invalid port number", type="error")
        return
    
    # Create the forward
    result = create_port_forward(
        session['id'],
        forward_type="local",
        local_port=local_port,
        remote_host=remote_host,
        remote_port=remote_port,
    )
    
    if result['success']:
        show_notification(
            f"Forwarding: localhost:{local_port} → {remote_host}:{remote_port}",
            type="success"
        )
    else:
        show_notification(f"Failed: {result.get('error', 'Unknown error')}", type="error")


@register_command("port-forward.list")
def list_forwards():
    """List all active port forwards."""
    session = get_active_session()
    if not session:
        show_notification("No active session", type="warning")
        return
    
    forwards = list_port_forwards(session['id'])
    
    if not forwards:
        show_notification("No active port forwards", type="info")
        return
    
    options = [
        f"{f['forward_type'].upper()}: localhost:{f['local_port']} → {f['remote_host']}:{f['remote_port']}"
        for f in forwards
    ]
    
    selected = show_quick_pick(
        options + ["[Cancel all forwards]"],
        placeholder="Select to remove, or cancel all..."
    )
    
    if not selected:
        return
    
    if selected == "[Cancel all forwards]":
        for f in forwards:
            remove_port_forward(session['id'], f['id'])
        show_notification("All port forwards removed", type="info")
    else:
        # Find and remove the selected forward
        for f in forwards:
            desc = f"{f['forward_type'].upper()}: localhost:{f['local_port']} → {f['remote_host']}:{f['remote_port']}"
            if desc == selected:
                remove_port_forward(session['id'], f['id'])
                show_notification(f"Removed: {desc}", type="info")
                break


@register_command("port-forward.save-preset")
def save_preset():
    """Save current forwards as a preset for this session."""
    session = get_active_session()
    if not session:
        show_notification("No active session", type="warning")
        return
    
    forwards = list_port_forwards(session['id'])
    
    if not forwards:
        show_notification("No active forwards to save", type="warning")
        return
    
    # Save to session storage
    set_session_var(session['id'], 'saved_forwards', forwards)
    show_notification(f"Saved {len(forwards)} port forward(s)", type="success")


@register_command("port-forward.restore-preset")
def restore_preset():
    """Restore saved port forwards for this session."""
    session = get_active_session()
    if not session:
        show_notification("No active session", type="warning")
        return
    
    saved = get_session_var(session['id'], 'saved_forwards')
    
    if not saved:
        show_notification("No saved forwards found", type="warning")
        return
    
    restored = 0
    for f in saved:
        result = create_port_forward(
            session['id'],
            forward_type=f.get('forward_type', 'local'),
            local_port=f['local_port'],
            remote_host=f['remote_host'],
            remote_port=f['remote_port'],
        )
        if result['success']:
            restored += 1
    
    show_notification(f"Restored {restored}/{len(saved)} port forward(s)", type="success")

