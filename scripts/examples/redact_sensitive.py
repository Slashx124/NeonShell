"""
@name: Redact Sensitive
@description: Redact sensitive patterns from terminal scrollback and output
@author: NeonShell Team
@version: 1.0.0
@hook: on_data
"""

from neonshell import (
    hook,
    register_command,
    show_notification,
    show_input,
    show_quick_pick,
    log,
    get_config,
    set_config,
    modify_output,
)
import re


# Default patterns to redact
DEFAULT_PATTERNS = {
    'aws_access_key': {
        'pattern': r'AKIA[0-9A-Z]{16}',
        'replacement': '[AWS_ACCESS_KEY_REDACTED]',
        'description': 'AWS Access Key IDs',
    },
    'aws_secret_key': {
        'pattern': r'(?<![A-Za-z0-9/+=])[A-Za-z0-9/+=]{40}(?![A-Za-z0-9/+=])',
        'replacement': '[AWS_SECRET_KEY_REDACTED]',
        'description': 'AWS Secret Access Keys',
    },
    'github_token': {
        'pattern': r'gh[pousr]_[A-Za-z0-9_]{36,}',
        'replacement': '[GITHUB_TOKEN_REDACTED]',
        'description': 'GitHub Personal Access Tokens',
    },
    'generic_api_key': {
        'pattern': r'(?i)(api[_-]?key|apikey)["\s:=]+["\']?([a-zA-Z0-9_-]{20,})["\']?',
        'replacement': r'\1=[API_KEY_REDACTED]',
        'description': 'Generic API Keys',
    },
    'password_in_url': {
        'pattern': r'(://[^:]+:)([^@]+)(@)',
        'replacement': r'\1[PASSWORD_REDACTED]\3',
        'description': 'Passwords in URLs',
    },
    'bearer_token': {
        'pattern': r'(Bearer\s+)[A-Za-z0-9_-]+\.?[A-Za-z0-9_-]*\.?[A-Za-z0-9_-]*',
        'replacement': r'\1[BEARER_TOKEN_REDACTED]',
        'description': 'Bearer tokens',
    },
    'jwt_token': {
        'pattern': r'eyJ[A-Za-z0-9_-]+\.eyJ[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+',
        'replacement': '[JWT_TOKEN_REDACTED]',
        'description': 'JWT Tokens',
    },
    'private_key': {
        'pattern': r'-----BEGIN (?:RSA |EC |DSA |OPENSSH )?PRIVATE KEY-----[\s\S]*?-----END (?:RSA |EC |DSA |OPENSSH )?PRIVATE KEY-----',
        'replacement': '[PRIVATE_KEY_REDACTED]',
        'description': 'Private Keys',
    },
    'ip_address': {
        'pattern': r'\b(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\b',
        'replacement': '[IP_REDACTED]',
        'description': 'IP Addresses',
        'enabled': False,  # Disabled by default
    },
    'email': {
        'pattern': r'\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b',
        'replacement': '[EMAIL_REDACTED]',
        'description': 'Email Addresses',
        'enabled': False,  # Disabled by default
    },
}


def get_patterns():
    """Get all redaction patterns (default + user-defined)."""
    user_patterns = get_config('redact_patterns', {})
    
    # Merge with defaults
    patterns = {}
    for name, info in DEFAULT_PATTERNS.items():
        patterns[name] = {**info}
        
    for name, info in user_patterns.items():
        if name in patterns:
            patterns[name].update(info)
        else:
            patterns[name] = info
    
    return patterns


def is_pattern_enabled(name, pattern_info):
    """Check if a pattern is enabled."""
    # Check user override first
    enabled_patterns = get_config('redact_enabled', None)
    if enabled_patterns is not None:
        return name in enabled_patterns
    
    # Use default setting
    return pattern_info.get('enabled', True)


@hook("on_data")
def redact_output(session, data):
    """
    Process terminal output and redact sensitive patterns.
    
    This hook runs on all terminal output and replaces
    sensitive patterns with redacted placeholders.
    """
    if not get_config('redaction_enabled', True):
        return data
    
    patterns = get_patterns()
    modified = data
    redaction_count = 0
    
    for name, info in patterns.items():
        if not is_pattern_enabled(name, info):
            continue
        
        try:
            pattern = re.compile(info['pattern'])
            matches = pattern.findall(modified)
            
            if matches:
                modified = pattern.sub(info['replacement'], modified)
                redaction_count += len(matches)
        except re.error as e:
            log(f"Redact: Invalid pattern '{name}': {e}")
    
    if redaction_count > 0:
        log(f"Redact: Replaced {redaction_count} sensitive pattern(s)")
    
    return modified


@register_command("redact.toggle")
def toggle_redaction():
    """Toggle redaction on/off."""
    enabled = get_config('redaction_enabled', True)
    set_config('redaction_enabled', not enabled)
    
    status = "disabled" if enabled else "enabled"
    show_notification(f"Redaction {status}", type="info")


@register_command("redact.configure")
def configure_patterns():
    """Configure which patterns to redact."""
    patterns = get_patterns()
    enabled = get_config('redact_enabled', list(patterns.keys()))
    
    options = []
    for name, info in patterns.items():
        is_enabled = is_pattern_enabled(name, info)
        checkbox = "✓" if is_enabled else "○"
        options.append(f"{checkbox} {name}: {info['description']}")
    
    options.append("---")
    options.append("Add custom pattern")
    
    selected = show_quick_pick(options, placeholder="Toggle patterns...")
    
    if not selected:
        return
    
    if selected == "Add custom pattern":
        add_custom_pattern()
        return
    
    if selected == "---":
        return
    
    # Toggle the selected pattern
    # Extract pattern name
    pattern_name = selected.split(' ', 1)[1].split(':')[0].strip()
    
    if pattern_name in enabled:
        enabled.remove(pattern_name)
        show_notification(f"Disabled: {pattern_name}", type="info")
    else:
        enabled.append(pattern_name)
        show_notification(f"Enabled: {pattern_name}", type="info")
    
    set_config('redact_enabled', enabled)


def add_custom_pattern():
    """Add a custom redaction pattern."""
    name = show_input(placeholder="Pattern name...", prompt="Name")
    if not name:
        return
    
    pattern = show_input(placeholder="Regex pattern...", prompt="Pattern")
    if not pattern:
        return
    
    # Validate pattern
    try:
        re.compile(pattern)
    except re.error as e:
        show_notification(f"Invalid regex: {e}", type="error")
        return
    
    replacement = show_input(
        placeholder="Replacement text...",
        prompt="Replacement",
    ) or f"[{name.upper()}_REDACTED]"
    
    description = show_input(
        placeholder="Description...",
        prompt="Description",
    ) or f"Custom pattern: {name}"
    
    # Save pattern
    user_patterns = get_config('redact_patterns', {})
    user_patterns[name] = {
        'pattern': pattern,
        'replacement': replacement,
        'description': description,
    }
    set_config('redact_patterns', user_patterns)
    
    # Enable the new pattern
    enabled = get_config('redact_enabled', list(get_patterns().keys()))
    if name not in enabled:
        enabled.append(name)
        set_config('redact_enabled', enabled)
    
    show_notification(f"Pattern '{name}' added", type="success")


@register_command("redact.test")
def test_redaction():
    """Test redaction patterns on sample text."""
    test_text = show_input(
        placeholder="Enter text to test redaction...",
        prompt="Test Text",
    )
    
    if not test_text:
        return
    
    patterns = get_patterns()
    result = test_text
    matches_found = []
    
    for name, info in patterns.items():
        if not is_pattern_enabled(name, info):
            continue
        
        try:
            pattern = re.compile(info['pattern'])
            matches = pattern.findall(result)
            
            if matches:
                matches_found.append(f"{name}: {len(matches)} match(es)")
                result = pattern.sub(info['replacement'], result)
        except re.error:
            pass
    
    if matches_found:
        lines = ["Patterns matched:", ""] + matches_found + ["", "Result:", result]
        show_notification('\n'.join(lines), type="info")
    else:
        show_notification("No patterns matched", type="info")

