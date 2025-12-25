"""
@name: Host Health Dashboard
@description: Display host health metrics and system status
@author: NeonShell Team
@version: 1.0.0
"""

from neonshell import (
    register_command,
    run_command,
    run_command_capture,
    show_notification,
    show_panel,
    log,
    get_active_session,
    create_widget,
)


@register_command("host-health.dashboard")
def show_dashboard():
    """Show a health dashboard for the current host."""
    session = get_active_session()
    if not session:
        show_notification("No active session", type="warning")
        return
    
    # Gather health metrics
    metrics = gather_metrics(session['id'])
    
    # Create dashboard panel
    panel = create_widget({
        'type': 'panel',
        'title': f"Health: {session['host']}",
        'content': format_dashboard(metrics),
    })
    
    show_panel(panel)


def gather_metrics(session_id):
    """Gather system metrics from the remote host."""
    metrics = {}
    
    # CPU load
    result = run_command_capture(session_id, "cat /proc/loadavg 2>/dev/null")
    if result['success']:
        parts = result['output'].strip().split()
        if len(parts) >= 3:
            metrics['cpu_load'] = {
                '1min': float(parts[0]),
                '5min': float(parts[1]),
                '15min': float(parts[2]),
            }
    
    # Memory usage
    result = run_command_capture(session_id, "free -b 2>/dev/null")
    if result['success']:
        lines = result['output'].strip().split('\n')
        for line in lines:
            if line.startswith('Mem:'):
                parts = line.split()
                if len(parts) >= 3:
                    total = int(parts[1])
                    used = int(parts[2])
                    metrics['memory'] = {
                        'total': total,
                        'used': used,
                        'percent': (used / total) * 100 if total > 0 else 0,
                    }
    
    # Disk usage
    result = run_command_capture(session_id, "df -B1 / 2>/dev/null | tail -1")
    if result['success']:
        parts = result['output'].strip().split()
        if len(parts) >= 5:
            total = int(parts[1])
            used = int(parts[2])
            metrics['disk'] = {
                'total': total,
                'used': used,
                'percent': (used / total) * 100 if total > 0 else 0,
            }
    
    # Uptime
    result = run_command_capture(session_id, "uptime -p 2>/dev/null || uptime")
    if result['success']:
        metrics['uptime'] = result['output'].strip()
    
    # Process count
    result = run_command_capture(session_id, "ps aux 2>/dev/null | wc -l")
    if result['success']:
        try:
            metrics['processes'] = int(result['output'].strip()) - 1  # Subtract header
        except:
            pass
    
    # Network connections
    result = run_command_capture(session_id, "ss -tun 2>/dev/null | wc -l")
    if result['success']:
        try:
            metrics['connections'] = int(result['output'].strip()) - 1  # Subtract header
        except:
            pass
    
    return metrics


def format_dashboard(metrics):
    """Format metrics as a dashboard string."""
    lines = []
    
    # CPU Load
    if 'cpu_load' in metrics:
        load = metrics['cpu_load']
        bar = create_bar(load['1min'] / 4 * 100)  # Assuming 4 cores as "full"
        lines.append(f"CPU Load: {bar} {load['1min']:.2f} / {load['5min']:.2f} / {load['15min']:.2f}")
    
    # Memory
    if 'memory' in metrics:
        mem = metrics['memory']
        bar = create_bar(mem['percent'])
        used_gb = mem['used'] / (1024**3)
        total_gb = mem['total'] / (1024**3)
        lines.append(f"Memory:   {bar} {used_gb:.1f}GB / {total_gb:.1f}GB ({mem['percent']:.1f}%)")
    
    # Disk
    if 'disk' in metrics:
        disk = metrics['disk']
        bar = create_bar(disk['percent'])
        used_gb = disk['used'] / (1024**3)
        total_gb = disk['total'] / (1024**3)
        lines.append(f"Disk /:   {bar} {used_gb:.1f}GB / {total_gb:.1f}GB ({disk['percent']:.1f}%)")
    
    # Uptime
    if 'uptime' in metrics:
        lines.append(f"Uptime:   {metrics['uptime']}")
    
    # Processes
    if 'processes' in metrics:
        lines.append(f"Processes: {metrics['processes']}")
    
    # Connections
    if 'connections' in metrics:
        lines.append(f"Network Connections: {metrics['connections']}")
    
    return '\n'.join(lines) if lines else "No metrics available"


def create_bar(percent, width=20):
    """Create a text-based progress bar."""
    percent = max(0, min(100, percent))
    filled = int(width * percent / 100)
    empty = width - filled
    
    # Use colored segments based on percentage
    if percent < 50:
        color = '🟢'
    elif percent < 80:
        color = '🟡'
    else:
        color = '🔴'
    
    return f"[{'█' * filled}{'░' * empty}]"


@register_command("host-health.quick-status")
def quick_status():
    """Show a quick notification with host status."""
    session = get_active_session()
    if not session:
        show_notification("No active session", type="warning")
        return
    
    metrics = gather_metrics(session['id'])
    
    parts = []
    
    if 'cpu_load' in metrics:
        parts.append(f"Load: {metrics['cpu_load']['1min']:.2f}")
    
    if 'memory' in metrics:
        parts.append(f"Mem: {metrics['memory']['percent']:.0f}%")
    
    if 'disk' in metrics:
        parts.append(f"Disk: {metrics['disk']['percent']:.0f}%")
    
    status = " | ".join(parts) if parts else "Unable to gather metrics"
    
    show_notification(f"{session['host']}: {status}", type="info")


@register_command("host-health.top")
def show_top():
    """Run htop or top on the remote host."""
    session = get_active_session()
    if not session:
        show_notification("No active session", type="warning")
        return
    
    # Try htop first, fall back to top
    run_command(session['id'], "htop 2>/dev/null || top")


@register_command("host-health.disk-usage")
def show_disk_usage():
    """Show disk usage breakdown."""
    session = get_active_session()
    if not session:
        show_notification("No active session", type="warning")
        return
    
    run_command(session['id'], "df -h && echo '' && du -sh /* 2>/dev/null | sort -hr | head -20")




