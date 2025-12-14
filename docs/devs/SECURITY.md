# Security Policy

## Supported Versions

Mamba is currently in active development. Security updates will be provided for the latest version only.

| Version | Supported          |
| ------- | ------------------ |
| Latest  | :white_check_mark: |
| Older   | :x:                |

---

## Reporting a Vulnerability

If you discover a security vulnerability in Mamba, please report it responsibly.

### How to Report

**DO NOT create a public GitHub issue for security vulnerabilities.**

Instead:

1. Email the maintainers directly (contact information will be provided once available)
2. Include a detailed description of the vulnerability
3. Provide steps to reproduce (if applicable)
4. Suggest a fix or mitigation (if you have one)

### What to Include

- **Type of vulnerability** (e.g., code injection, memory safety, etc.)
- **Affected component** (parser, transpiler, CLI, etc.)
- **Impact assessment** (potential damage or exploit scenarios)
- **Reproduction steps** with example code
- **Environment details** (OS, Mamba version)

---

## Response Timeline

- **Acknowledgment:** Within 48 hours
- **Initial assessment:** Within 7 days
- **Fix and disclosure:** Varies by severity

We will keep you informed throughout the process.

---

## Security Best Practices

### For Users

- **Keep Mamba updated** to the latest version
- **Verify source integrity** when downloading
- **Review generated code** in sensitive environments
- **Be cautious with untrusted input** files

### For Contributors

- **Follow secure coding practices** in Rust
- **Avoid unsafe code** unless absolutely necessary (and document why)
- **Validate all user input** thoroughly
- **Use Rust's safety guarantees** (ownership, borrowing, lifetimes)
- **Review dependencies** for known vulnerabilities
- **Run `cargo audit`** before major releases

---

## Known Security Considerations

### Bundled Rust Toolchain

- Mamba ships with an embedded Rust compiler
- The bundled toolchain is regularly updated
- Users should ensure they download Mamba from official sources only

### Code Generation

- Generated Rust code follows safe patterns
- Avoid `unsafe` blocks in transpiler output
- All memory management is handled by Rust's ownership system

### Compiler Security

- Rust compiler vulnerabilities are inherited by Mamba
- Monitor Rust security advisories
- Update bundled toolchain when necessary

---

## Disclosure Policy

We follow a **coordinated disclosure** approach:

1. Vulnerability is reported privately
2. Issue is confirmed and assessed
3. Fix is developed and tested
4. Security advisory is prepared
5. Fix is released
6. Public disclosure is made

We aim to release fixes before public disclosure, typically within 90 days.

---

## Security Advisories

Security advisories will be published in:

- GitHub Security Advisories
- Release notes
- Project documentation

---

## Questions?

For security-related questions (non-vulnerabilities), you can:

- Open a GitHub Discussion
- Check existing documentation
- Contact maintainers

---

Thank you for helping keep Mamba secure! 🛡️
