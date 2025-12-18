"""
@name: Auto Tmux
@description: Automatically attach to or create a tmux session on connect
@author: NeonShell Team
@version: 1.0.0
@hook: on_connect
"""

from neonshell import hook, run_command, log, get_session_var, set_session_var


@hook("on_connect")
def attach_tmux(session):
    """
    Automatically attach to an existing tmux session or create a new one.
    
    If a tmux session named 'main' exists, attach to it.
    Otherwise, create a new tmux session named 'main'.
    """
    log(f"Auto-tmux: Checking for tmux session on {session['host']}")
    
    # Check if we should skip (user might have disabled this per-host)
    if get_session_var(session['id'], 'skip_tmux'):
        log("Auto-tmux: Skipped (disabled for this session)")
        return
    
    # Default tmux session name
    session_name = get_session_var(session['id'], 'tmux_session_name') or 'main'
    
    # Try to attach to existing session, or create a new one
    command = f"tmux attach -t {session_name} 2>/dev/null || tmux new -s {session_name}"
    
    log(f"Auto-tmux: Running: {command}")
    run_command(session['id'], command)
    
    # Mark that we've attached tmux for this session
    set_session_var(session['id'], 'tmux_attached', True)


@hook("on_disconnect")
def cleanup_tmux(session):
    """Clean up tmux state on disconnect."""
    if get_session_var(session['id'], 'tmux_attached'):
        log(f"Auto-tmux: Session {session['host']} disconnected (tmux was attached)")


def configure(session_id, session_name='main', enabled=True):
    """
    Configure auto-tmux for a specific session.
    
    Args:
        session_id: The session ID to configure
        session_name: The tmux session name to use (default: 'main')
        enabled: Whether auto-tmux is enabled for this session
    """
    set_session_var(session_id, 'tmux_session_name', session_name)
    set_session_var(session_id, 'skip_tmux', not enabled)
    log(f"Auto-tmux configured: session_name={session_name}, enabled={enabled}")

