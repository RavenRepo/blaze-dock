# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

We take the security of BlazeDock seriously. If you have discovered a security vulnerability, we appreciate your help in disclosing it to us in a responsible manner.

### How to Report

**Please do NOT report security vulnerabilities through public GitHub issues.**

Instead, please report them via one of the following methods:

1. **GitHub Security Advisories**: Use GitHub's private vulnerability reporting feature at [Security Advisories](https://github.com/RavenRepo/blaze-dock/security/advisories/new)

2. **Email**: Contact the maintainers directly (if contact information is available in the repository)

### What to Include

Please include the following information in your report:

- Type of vulnerability (e.g., buffer overflow, privilege escalation, code injection)
- Full paths of source file(s) related to the vulnerability
- Location of the affected source code (tag/branch/commit or direct URL)
- Step-by-step instructions to reproduce the issue
- Proof-of-concept or exploit code (if possible)
- Impact of the vulnerability

### Response Timeline

- **Initial Response**: Within 48 hours
- **Status Update**: Within 7 days
- **Resolution Target**: Within 90 days (depending on complexity)

### What to Expect

1. **Acknowledgment**: We will acknowledge receipt of your vulnerability report
2. **Communication**: We will keep you informed of the progress
3. **Credit**: If you wish, we will credit you in the security advisory and release notes
4. **Fix**: We will work to fix the vulnerability and release a patched version

## Security Best Practices

### For Users

- Always download BlazeDock from official sources (GitHub releases)
- Keep your system and dependencies updated
- Report any suspicious behavior

### For Contributors

- Follow secure coding practices
- Never commit sensitive data (credentials, API keys)
- Use the latest stable Rust toolchain
- Run `cargo audit` to check for known vulnerabilities in dependencies

## Scope

This security policy applies to:

- The BlazeDock application code
- Configuration file handling
- D-Bus interface (when implemented)
- Plugin/extension system (when implemented)

### Out of Scope

- Vulnerabilities in third-party dependencies (report to upstream)
- Issues in the Wayland compositor
- GTK4 or system library vulnerabilities

## Security Features

BlazeDock is designed with security in mind:

- **Memory Safety**: Written in Rust, preventing common memory vulnerabilities
- **Minimal Privileges**: Runs with user-level permissions only
- **No Network Access**: Core functionality doesn't require network (except app launches)
- **Sandboxed Processes**: Launched applications run in their own process space

---

Thank you for helping keep BlazeDock and its users safe!

