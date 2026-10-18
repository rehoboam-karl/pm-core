# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | ✅ Currently used in production |

## Reporting a Vulnerability

Do **not** open a public GitHub issue for security vulnerabilities.

Email the author directly: see GitHub profile.

Include as much detail as possible:
- Description of the vulnerability
- Steps to reproduce
- Potential impact
- Suggested fix (if any)

## Security Model

`pm-core` is a **decision library only**. It does not:
- Hold private keys
- Sign transactions
- Connect to networks directly
- Store credentials

Your trading engine is responsible for:
- Secure key management
- Network security
- Input validation on external data (prices, API responses)
- Rate limiting and circuit breaker persistence

## Design notes

- All public APIs validate inputs — don't panic on bad data
- `RiskGuard` state is in-memory only; persist if you need restart resilience
- `TradeJournal` writes to disk — ensure the data directory is properly secured
- No hardcoded credentials; all config via environment / constructor args
