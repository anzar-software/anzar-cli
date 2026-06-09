## Anzar CLI

Anzar is a lightweight authentication and authorization framework designed to run as a separate microservice. It provides tools to initialize, configure, and manage your project’s authentication system with ease.

## Features
* Lightweight and modular
* Easy setup for any project
* Database schema generation and migration support
* Service status monitoring
* Simple CLI commands for developers

---

## Installation
Install Anzar globally using npm (or your preferred package manager):
```bash
npm install -g @anzar-auth/anzar
```

---

## Usage
Anzar provides a command-line interface to manage your authentication service.
```bash
anzar <COMMAND> [OPTIONS]
```

### Commands

| Command       |  Description                                      |
|---------------|---------------------------------------------------|
| `init`        |  Initialize Anzar for your project                                 |
| `check`       |  Check current configuration and setup                             |
| `status`      |  Show Anzar service status                                         |
| `generate`    |  Generate database schemas                                         |
| `migrate`     |  Apply database migrations                                         |
| `help`        |  Print this message or the help of a subcommand                    |

### Options
| Option           |  Description                                      |
|------------------|---------------------------------------------------|
| `-h, --help`     | Print help                                 |
| `-V, --version`  |   Print version                                |

### Examples
Initialize a new Anzar project:
```bash
anzar init
```

Check your current configuration:
```bash
anzar check
```

Generate database schemas:
```bash
anzar generate
```
Apply pending migrations:
```bash
anzar migrate
```

Show the current service status:
```bash
anzar status
```

Get help for a specific command:
```bash
anzar help migrate
```






