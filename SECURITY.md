# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

If you discover a security vulnerability in NeonShell, please report it responsibly:

1. **Do not** open a public GitHub issue
2. Email security@neonshell.dev (replace with your actual security email)
3. Include as much detail as possible:
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact
   - Suggested fix (if any)

We will acknowledge receipt within 48 hours and provide a detailed response within 7 days.

## Security Model

### Threat Model

NeonShell handles sensitive information including SSH credentials and private keys. Our security model addresses the following threats:

| Threat | Mitigation |
|--------|------------|
| **Private key theft** | Keys stored encrypted in OS keychain (macOS Keychain, Windows Credential Manager, Linux Secret Service). Never stored in plaintext on disk. |
| **Credential logging** | All secrets are filtered from logs. Debug output is sanitized. Secret values are never included in error messages. |
| **Memory disclosure** | Rust's memory safety prevents buffer overflows. Secrets are zeroed after use where possible. |
| **Malicious plugins** | Plugins run in a sandboxed environment with explicit permission grants. Users are warned about unsigned plugins. |
| **Malicious scripts** | Python scripts run in a separate subprocess with limited IPC capabilities. Scripts must request permissions. |
| **Man-in-the-middle** | Strict known_hosts policy by default. Clear fingerprint UI. Certificate pinning supported. |
| **Local privilege escalation** | Minimal use of elevated privileges. All sensitive operations go through OS security APIs. |

### Secret Storage

```
┌─────────────────────────────────────────────────────────────┐
│                        USER INPUT                            │
│  (passwords, private keys, passphrases)                     │
└────────────────────────────┬────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────┐
│                     NEONSHELL APP                           │
│  - Secrets held only in memory during use                   │
│  - Never written to logs or disk                            │
│  - Zeroed after authentication completes                    │
└────────────────────────────┬────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────┐
│                    OS KEYCHAIN                              │
│  macOS: Keychain Services                                   │
│  Windows: Credential Manager                                │
│  Linux: libsecret (Secret Service API)                     │
│                                                             │
│  - Encrypted at rest by OS                                  │
│  - Access controlled by OS security                         │
│  - Survives app reinstallation                             │
└─────────────────────────────────────────────────────────────┘
```

### Known Hosts Policy

NeonShell implements strict known_hosts handling:

1. **Strict (default)**: Reject connections to unknown hosts. Alert on fingerprint changes.
2. **Ask**: Prompt user to verify fingerprint for unknown hosts.
3. **Accept**: Auto-accept new hosts (not recommended).

When a host key changes, NeonShell will:
- Show a prominent warning with old and new fingerprints
- Require explicit user confirmation to proceed
- Log the event (without the key itself)

### Plugin Security

Plugins are sandboxed with the following controls:

1. **Permission System**: Plugins must declare required permissions in manifest
2. **User Approval**: Users must approve permissions before plugin activation
3. **Capability Restriction**: Plugins can only access APIs matching their permissions
4. **Signature Verification**: Optional plugin signing with warnings for unsigned plugins

Available permissions:
- `network`: Make HTTP requests
- `filesystem`: Read/write to plugin data directory only
- `clipboard`: Read/write clipboard
- `notifications`: Show system notifications
- `terminal`: Read terminal output
- `shell`: Execute shell commands (dangerous)

### Python Script Security

Python scripts run in a separate subprocess:

1. Scripts communicate via JSON-RPC over stdin/stdout
2. Limited API surface exposed through the `neonshell` module
3. No direct access to keychain or raw SSH sessions
4. Scripts can be enabled/disabled per-profile

### Logging Policy

NeonShell logs are designed to aid debugging without exposing secrets:

**Never logged:**
- Passwords
- Private key contents
- Passphrases
- Authentication tokens
- Session cookies

**Always logged (if debug enabled):**
- Connection attempts (host, port, username)
- Authentication method used (not credentials)
- Session lifecycle events
- Plugin/script activation
- Error messages (sanitized)

### Audit Log

NeonShell maintains an audit log of security-relevant events:
- Connection attempts and results
- Known hosts changes
- Keychain access
- Plugin permission grants
- Script executions

### Best Practices for Users

1. **Use SSH Agent**: Prefer ssh-agent over storing keys directly
2. **Enable 2FA**: Use hardware keys or TOTP where possible
3. **Verify Fingerprints**: Always verify host fingerprints on first connect
4. **Review Plugins**: Only install plugins from trusted sources
5. **Review Scripts**: Understand what scripts do before enabling them
6. **Keep Updated**: Install security updates promptly
7. **Lock Screen**: Enable auto-lock to protect sessions when away

### Vulnerability Disclosure

We follow responsible disclosure practices:

1. Report received and acknowledged (48 hours)
2. Vulnerability confirmed and assessed (7 days)
3. Fix developed and tested (varies by severity)
4. Security advisory published with fix
5. CVE assigned if applicable

### Security Updates

Security updates are released as soon as possible after a fix is ready:

- **Critical**: Immediate patch release
- **High**: Within 7 days
- **Medium**: Next scheduled release
- **Low**: Backlog for future release

## Contact

For security concerns: security@neonshell.dev

For general questions: Open a GitHub discussion

