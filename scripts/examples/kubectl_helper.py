"""
@name: Kubectl Helper
@description: Kubernetes context management and common commands via SSH
@author: NeonShell Team
@version: 1.0.0
"""

from neonshell import (
    register_command,
    run_command,
    run_command_capture,
    show_quick_pick,
    show_notification,
    log,
    get_active_session,
)


@register_command("kubectl.switch-context")
def switch_context():
    """Switch Kubernetes context on the remote host."""
    session = get_active_session()
    if not session:
        show_notification("No active session", type="warning")
        return
    
    # Get available contexts
    result = run_command_capture(session['id'], "kubectl config get-contexts -o name")
    
    if not result['success']:
        show_notification("Failed to get contexts", type="error")
        return
    
    contexts = [c.strip() for c in result['output'].split('\n') if c.strip()]
    
    if not contexts:
        show_notification("No contexts found", type="warning")
        return
    
    # Show context picker
    selected = show_quick_pick(contexts, placeholder="Select context...")
    
    if selected:
        run_command(session['id'], f"kubectl config use-context {selected}")
        show_notification(f"Switched to context: {selected}", type="success")


@register_command("kubectl.switch-namespace")
def switch_namespace():
    """Switch default namespace on the remote host."""
    session = get_active_session()
    if not session:
        show_notification("No active session", type="warning")
        return
    
    # Get available namespaces
    result = run_command_capture(session['id'], "kubectl get namespaces -o name")
    
    if not result['success']:
        show_notification("Failed to get namespaces", type="error")
        return
    
    namespaces = [
        n.replace('namespace/', '').strip() 
        for n in result['output'].split('\n') 
        if n.strip()
    ]
    
    if not namespaces:
        show_notification("No namespaces found", type="warning")
        return
    
    # Show namespace picker
    selected = show_quick_pick(namespaces, placeholder="Select namespace...")
    
    if selected:
        run_command(
            session['id'],
            f"kubectl config set-context --current --namespace={selected}"
        )
        show_notification(f"Switched to namespace: {selected}", type="success")


@register_command("kubectl.get-pods")
def get_pods():
    """Get pods in current namespace."""
    session = get_active_session()
    if not session:
        show_notification("No active session", type="warning")
        return
    
    run_command(session['id'], "kubectl get pods -o wide")


@register_command("kubectl.pod-logs")
def pod_logs():
    """Tail logs from a pod."""
    session = get_active_session()
    if not session:
        show_notification("No active session", type="warning")
        return
    
    # Get available pods
    result = run_command_capture(session['id'], "kubectl get pods -o name")
    
    if not result['success']:
        show_notification("Failed to get pods", type="error")
        return
    
    pods = [
        p.replace('pod/', '').strip() 
        for p in result['output'].split('\n') 
        if p.strip()
    ]
    
    if not pods:
        show_notification("No pods found", type="warning")
        return
    
    selected = show_quick_pick(pods, placeholder="Select pod...")
    
    if selected:
        run_command(session['id'], f"kubectl logs -f {selected}")


@register_command("kubectl.exec-shell")
def exec_shell():
    """Execute a shell in a pod."""
    session = get_active_session()
    if not session:
        show_notification("No active session", type="warning")
        return
    
    # Get available pods
    result = run_command_capture(session['id'], "kubectl get pods -o name")
    
    if not result['success']:
        show_notification("Failed to get pods", type="error")
        return
    
    pods = [
        p.replace('pod/', '').strip() 
        for p in result['output'].split('\n') 
        if p.strip()
    ]
    
    if not pods:
        show_notification("No pods found", type="warning")
        return
    
    selected = show_quick_pick(pods, placeholder="Select pod to exec into...")
    
    if selected:
        # Try bash first, fall back to sh
        run_command(
            session['id'],
            f"kubectl exec -it {selected} -- /bin/bash || kubectl exec -it {selected} -- /bin/sh"
        )


@register_command("kubectl.port-forward")
def port_forward():
    """Set up port forwarding to a pod or service."""
    session = get_active_session()
    if not session:
        show_notification("No active session", type="warning")
        return
    
    # Get pods and services
    result_pods = run_command_capture(session['id'], "kubectl get pods -o name")
    result_svc = run_command_capture(session['id'], "kubectl get svc -o name")
    
    items = []
    
    if result_pods['success']:
        items.extend([
            f"pod/{p.replace('pod/', '').strip()}"
            for p in result_pods['output'].split('\n')
            if p.strip()
        ])
    
    if result_svc['success']:
        items.extend([
            f"svc/{s.replace('service/', '').strip()}"
            for s in result_svc['output'].split('\n')
            if s.strip()
        ])
    
    if not items:
        show_notification("No pods or services found", type="warning")
        return
    
    selected = show_quick_pick(items, placeholder="Select resource...")
    
    if selected:
        # Would show input dialog for port mapping
        # For now, use a default
        local_port = 8080
        remote_port = 80
        
        log(f"kubectl: Port forwarding {selected} {local_port}:{remote_port}")
        run_command(
            session['id'],
            f"kubectl port-forward {selected} {local_port}:{remote_port}"
        )
        show_notification(
            f"Port forwarding: localhost:{local_port} -> {selected}:{remote_port}",
            type="info"
        )


@register_command("kubectl.describe")
def describe_resource():
    """Describe a Kubernetes resource."""
    session = get_active_session()
    if not session:
        show_notification("No active session", type="warning")
        return
    
    resource_types = [
        "pods",
        "services",
        "deployments",
        "configmaps",
        "secrets",
        "ingresses",
        "nodes",
    ]
    
    resource_type = show_quick_pick(resource_types, placeholder="Select resource type...")
    
    if not resource_type:
        return
    
    # Get resources of that type
    result = run_command_capture(session['id'], f"kubectl get {resource_type} -o name")
    
    if not result['success']:
        show_notification(f"Failed to get {resource_type}", type="error")
        return
    
    resources = [r.strip() for r in result['output'].split('\n') if r.strip()]
    
    if not resources:
        show_notification(f"No {resource_type} found", type="warning")
        return
    
    selected = show_quick_pick(resources, placeholder=f"Select {resource_type}...")
    
    if selected:
        run_command(session['id'], f"kubectl describe {selected}")




