"""
@name: Bastion Rotator
@description: Rotate through multiple bastion/jump hosts for load balancing or failover
@author: NeonShell Team
@version: 1.0.0
@hook: on_connect
"""

from neonshell import (
    hook,
    register_command,
    show_quick_pick,
    show_notification,
    show_input,
    log,
    get_config,
    set_config,
    get_active_session,
    modify_connection,
    test_connection,
)
import random


def get_bastion_groups():
    """Get configured bastion groups."""
    return get_config('bastion_groups', {})


def set_bastion_groups(groups):
    """Save bastion groups."""
    set_config('bastion_groups', groups)


@register_command("bastion.configure-group")
def configure_group():
    """Configure a bastion group."""
    groups = get_bastion_groups()
    
    options = list(groups.keys()) + ["+ Create new group"]
    selected = show_quick_pick(options, placeholder="Select or create a bastion group...")
    
    if not selected:
        return
    
    if selected == "+ Create new group":
        group_name = show_input(placeholder="Group name (e.g., prod-bastions)...", prompt="Name")
        if not group_name:
            return
        groups[group_name] = {
            'hosts': [],
            'strategy': 'round-robin',
            'health_check': True,
        }
    else:
        group_name = selected
    
    # Edit the group
    group = groups[group_name]
    
    action = show_quick_pick([
        "Add host",
        "Remove host",
        "Set strategy",
        "Toggle health check",
        "View hosts",
        "Delete group",
    ], placeholder="Select action...")
    
    if not action:
        return
    
    if action == "Add host":
        host_str = show_input(
            placeholder="user@host:port (e.g., admin@bastion1.example.com:22)...",
            prompt="Bastion Host",
        )
        if host_str:
            # Parse host string
            host_info = parse_host_string(host_str)
            if host_info:
                group['hosts'].append(host_info)
                show_notification(f"Added {host_str} to {group_name}", type="success")
    
    elif action == "Remove host":
        if not group['hosts']:
            show_notification("No hosts in this group", type="info")
            return
        
        host_options = [
            f"{h.get('user', 'unknown')}@{h['host']}:{h.get('port', 22)}"
            for h in group['hosts']
        ]
        to_remove = show_quick_pick(host_options, placeholder="Select host to remove...")
        
        if to_remove:
            idx = host_options.index(to_remove)
            group['hosts'].pop(idx)
            show_notification(f"Removed {to_remove}", type="info")
    
    elif action == "Set strategy":
        strategy = show_quick_pick(
            ["round-robin", "random", "failover", "health-based"],
            placeholder="Select rotation strategy...",
        )
        if strategy:
            group['strategy'] = strategy
            show_notification(f"Strategy set to {strategy}", type="success")
    
    elif action == "Toggle health check":
        group['health_check'] = not group.get('health_check', True)
        status = "enabled" if group['health_check'] else "disabled"
        show_notification(f"Health check {status}", type="info")
    
    elif action == "View hosts":
        if group['hosts']:
            lines = [f"Bastion Group: {group_name}", f"Strategy: {group['strategy']}", ""]
            for h in group['hosts']:
                lines.append(f"  - {h.get('user', '?')}@{h['host']}:{h.get('port', 22)}")
            show_notification('\n'.join(lines), type="info")
        else:
            show_notification("No hosts configured", type="info")
        return  # Don't save
    
    elif action == "Delete group":
        del groups[group_name]
        show_notification(f"Deleted group {group_name}", type="info")
    
    set_bastion_groups(groups)


def parse_host_string(host_str):
    """Parse user@host:port format."""
    try:
        user = None
        port = 22
        
        if '@' in host_str:
            user, host_str = host_str.split('@', 1)
        
        if ':' in host_str:
            host, port_str = host_str.rsplit(':', 1)
            port = int(port_str)
        else:
            host = host_str
        
        return {
            'host': host,
            'port': port,
            'user': user,
        }
    except:
        return None


@register_command("bastion.select-next")
def select_next_bastion():
    """Manually select the next bastion to use."""
    groups = get_bastion_groups()
    
    if not groups:
        show_notification("No bastion groups configured", type="warning")
        return
    
    # Select group
    group_name = show_quick_pick(list(groups.keys()), placeholder="Select bastion group...")
    
    if not group_name:
        return
    
    group = groups[group_name]
    
    if not group['hosts']:
        show_notification("No hosts in this group", type="warning")
        return
    
    # Get next bastion based on strategy
    bastion = get_next_bastion(group_name, group)
    
    if bastion:
        show_notification(
            f"Selected: {bastion.get('user', '')}@{bastion['host']}:{bastion.get('port', 22)}",
            type="success"
        )
        
        # Store selection for next connection
        set_config('next_bastion', bastion)


def get_next_bastion(group_name, group):
    """Get the next bastion host based on strategy."""
    hosts = group['hosts']
    strategy = group.get('strategy', 'round-robin')
    
    if not hosts:
        return None
    
    # Health check if enabled
    if group.get('health_check'):
        healthy_hosts = []
        for host in hosts:
            if check_host_health(host):
                healthy_hosts.append(host)
        
        if not healthy_hosts:
            log("Bastion: All hosts failed health check, using all hosts")
            healthy_hosts = hosts
        
        hosts = healthy_hosts
    
    if strategy == 'random':
        return random.choice(hosts)
    
    elif strategy == 'failover':
        # Always use first healthy host
        return hosts[0]
    
    elif strategy == 'round-robin':
        # Track last used index
        counters = get_config('bastion_counters', {})
        idx = counters.get(group_name, 0)
        
        bastion = hosts[idx % len(hosts)]
        
        counters[group_name] = (idx + 1) % len(hosts)
        set_config('bastion_counters', counters)
        
        return bastion
    
    # Default: first host
    return hosts[0]


def check_host_health(host_info):
    """Check if a bastion host is reachable."""
    result = test_connection(
        host=host_info['host'],
        port=host_info.get('port', 22),
        timeout=5,
    )
    return result.get('success', False)


@hook("on_connect")
def apply_bastion_rotation(session):
    """Apply bastion rotation if configured for this profile."""
    profile_id = session.get('profile_id')
    
    if not profile_id:
        return
    
    # Check if this profile uses bastion rotation
    profile_bastion = get_config(f'profile_bastion.{profile_id}')
    
    if not profile_bastion:
        return
    
    groups = get_bastion_groups()
    group = groups.get(profile_bastion)
    
    if not group:
        log(f"Bastion group '{profile_bastion}' not found")
        return
    
    bastion = get_next_bastion(profile_bastion, group)
    
    if bastion:
        log(f"Bastion: Using {bastion['host']} for connection")
        modify_connection(session['id'], {
            'jump_hosts': [{
                'host': bastion['host'],
                'port': bastion.get('port', 22),
                'username': bastion.get('user') or session.get('username'),
                'auth_method': {'type': 'agent'},
            }],
        })


@register_command("bastion.assign-to-profile")
def assign_to_profile():
    """Assign a bastion group to a connection profile."""
    # This would integrate with the profile system
    show_notification("Open profile settings to assign bastion groups", type="info")

