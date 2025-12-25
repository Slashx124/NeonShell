"""
@name: Snippet Expander
@description: Expand command snippets with variable substitution
@author: NeonShell Team
@version: 1.0.0
@hook: on_command
"""

from neonshell import (
    hook,
    register_command,
    run_command,
    show_quick_pick,
    show_input,
    show_notification,
    log,
    get_active_session,
    get_config,
    set_config,
)
import re


# Default snippets
DEFAULT_SNIPPETS = {
    "docker-logs": {
        "template": "docker logs -f --tail ${lines:100} ${container}",
        "description": "Follow Docker container logs",
    },
    "find-large": {
        "template": "find ${path:/} -type f -size +${size:100M} 2>/dev/null | head -${limit:20}",
        "description": "Find files larger than size",
    },
    "grep-recursive": {
        "template": "grep -rn '${pattern}' ${path:.} --include='${glob:*}'",
        "description": "Recursive grep with pattern",
    },
    "tar-extract": {
        "template": "tar -${flags:xzvf} ${archive} -C ${dest:.}",
        "description": "Extract tar archive",
    },
    "disk-hogs": {
        "template": "du -sh ${path:/*} 2>/dev/null | sort -hr | head -${limit:20}",
        "description": "Find disk space hogs",
    },
    "watch-proc": {
        "template": "watch -n ${interval:1} 'ps aux | grep ${process} | grep -v grep'",
        "description": "Watch a process",
    },
    "port-check": {
        "template": "nc -zv ${host:localhost} ${port}",
        "description": "Check if port is open",
    },
    "curl-time": {
        "template": "curl -w 'Time: %{time_total}s\\n' -o /dev/null -s ${url}",
        "description": "Time a URL request",
    },
    "mysql-query": {
        "template": "mysql -u ${user:root} -p -e '${query}' ${database}",
        "description": "Run MySQL query",
    },
    "pg-query": {
        "template": "psql -U ${user:postgres} -d ${database} -c '${query}'",
        "description": "Run PostgreSQL query",
    },
}


def get_snippets():
    """Get all snippets (user + defaults)."""
    user_snippets = get_config('snippets', {})
    return {**DEFAULT_SNIPPETS, **user_snippets}


def parse_variables(template):
    """
    Parse variables from a template.
    Format: ${name} or ${name:default}
    """
    pattern = r'\$\{(\w+)(?::([^}]*))?\}'
    matches = re.findall(pattern, template)
    return [(name, default or '') for name, default in matches]


def expand_template(template, values):
    """Expand a template with given values."""
    result = template
    
    for name, value in values.items():
        # Replace ${name:default} and ${name}
        result = re.sub(
            rf'\$\{{{name}(?::[^}}]*)?\}}',
            value,
            result
        )
    
    return result


@register_command("snippets.expand")
def expand_snippet():
    """Select and expand a snippet."""
    session = get_active_session()
    if not session:
        show_notification("No active session", type="warning")
        return
    
    snippets = get_snippets()
    
    # Build options with descriptions
    options = [
        f"{name}: {info['description']}"
        for name, info in snippets.items()
    ]
    
    selected = show_quick_pick(options, placeholder="Select snippet...")
    
    if not selected:
        return
    
    # Extract snippet name
    snippet_name = selected.split(':')[0]
    snippet = snippets.get(snippet_name)
    
    if not snippet:
        show_notification("Snippet not found", type="error")
        return
    
    template = snippet['template']
    variables = parse_variables(template)
    
    # Gather variable values
    values = {}
    for name, default in variables:
        value = show_input(
            placeholder=f"{name} (default: {default})" if default else name,
            prompt=name,
        )
        values[name] = value if value else default
    
    # Expand and run
    command = expand_template(template, values)
    log(f"Snippet expanded: {command}")
    run_command(session['id'], command)


@register_command("snippets.add")
def add_snippet():
    """Add a new custom snippet."""
    name = show_input(placeholder="Snippet name (e.g., my-command)...", prompt="Name")
    if not name:
        return
    
    template = show_input(
        placeholder="Command template (use ${var} or ${var:default})...",
        prompt="Template",
    )
    if not template:
        return
    
    description = show_input(placeholder="Description...", prompt="Description") or ""
    
    # Save snippet
    user_snippets = get_config('snippets', {})
    user_snippets[name] = {
        'template': template,
        'description': description,
    }
    set_config('snippets', user_snippets)
    
    show_notification(f"Snippet '{name}' saved", type="success")


@register_command("snippets.edit")
def edit_snippet():
    """Edit an existing snippet."""
    snippets = get_snippets()
    user_snippets = get_config('snippets', {})
    
    # Only allow editing user snippets
    if not user_snippets:
        show_notification("No custom snippets to edit", type="info")
        return
    
    options = list(user_snippets.keys())
    selected = show_quick_pick(options, placeholder="Select snippet to edit...")
    
    if not selected or selected not in user_snippets:
        return
    
    snippet = user_snippets[selected]
    
    new_template = show_input(
        placeholder=snippet['template'],
        prompt="Template",
    ) or snippet['template']
    
    new_description = show_input(
        placeholder=snippet.get('description', ''),
        prompt="Description",
    ) or snippet.get('description', '')
    
    user_snippets[selected] = {
        'template': new_template,
        'description': new_description,
    }
    set_config('snippets', user_snippets)
    
    show_notification(f"Snippet '{selected}' updated", type="success")


@register_command("snippets.delete")
def delete_snippet():
    """Delete a custom snippet."""
    user_snippets = get_config('snippets', {})
    
    if not user_snippets:
        show_notification("No custom snippets to delete", type="info")
        return
    
    options = list(user_snippets.keys())
    selected = show_quick_pick(options, placeholder="Select snippet to delete...")
    
    if selected and selected in user_snippets:
        del user_snippets[selected]
        set_config('snippets', user_snippets)
        show_notification(f"Snippet '{selected}' deleted", type="success")


@register_command("snippets.list")
def list_snippets():
    """List all available snippets."""
    snippets = get_snippets()
    user_snippets = get_config('snippets', {})
    
    lines = ["Available Snippets:", ""]
    
    for name, info in sorted(snippets.items()):
        marker = "[user]" if name in user_snippets else "[built-in]"
        lines.append(f"  {name} {marker}")
        lines.append(f"    {info['description']}")
        lines.append(f"    Template: {info['template']}")
        lines.append("")
    
    show_notification('\n'.join(lines), type="info")




