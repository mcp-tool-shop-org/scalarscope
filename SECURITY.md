# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 3.1.x (display version 3.1.1) | :white_check_mark: |
| 3.0.x (never released)        | :x:                |
| < 3.0   | :x:                |

## Reporting a Vulnerability

We take security seriously. If you discover a security vulnerability in ScalarScope, please report it responsibly.

### How to Report

1. **DO NOT** create a public GitHub issue for security vulnerabilities
2. Report privately at https://github.com/mcp-tool-shop-org/scalarscope/security/advisories/new
3. Include:
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact
   - Any suggested fixes (optional)

### What to Expect

- **Acknowledgment**: Within 48 hours
- **Initial Assessment**: Within 7 days
- **Resolution Timeline**: Depends on severity
  - Critical: 24-72 hours
  - High: 7 days
  - Medium: 30 days
  - Low: Next release cycle

### Scope

This security policy covers:
- ScalarScope application (ScalarScope.csproj)
- VortexKit visualization library
- Data handling and storage
- Export functionality

### Out of Scope

- Third-party dependencies (report to their maintainers)
- Issues already publicly disclosed
- Social engineering attacks

## Security Best Practices

When using ScalarScope:

1. **Data Files**: Only open training run JSON files from trusted sources
2. **Exports**: Review exported images/data before sharing
3. **Updates**: Keep the application updated to the latest version

## Acknowledgments

We appreciate responsible disclosure and will acknowledge security researchers who report valid vulnerabilities (unless you prefer to remain anonymous).
