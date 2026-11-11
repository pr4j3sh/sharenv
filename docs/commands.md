# Command reference

Flags accept both space-separated and `=` forms: `--project-id <id>`,
`--project-id=<id>`, `--role <role>`, `--role=<role>`.

Global options (per invocation, like `--insecure`): `--profile <name>` selects a
named keyring slot instead of the per-server default.

Primary entities are positional arguments; subcommands disambiguate; flags are
modifiers only. The old `-e`/`-p`/`-pid` flags are gone (hard break).

| Command    | Alias               | Flags                                                    | Description                                                                                 |
| ---------- | ------------------- | -------------------------------------------------------- | ------------------------------------------------------------------------------------------- |
| `register` | `r`                 | —                                                        | Register a new account.                                                                     |
| `login`    | `l`                 | —                                                        | Log in.                                                                                     |
| `logout`   | —                   | —                                                        | Revoke the server-side session and remove local tokens.                                      |
| `verify`   | —                   | `[email]`                                                | Verify an account with the emailed code (defaults to the email saved at registration). Saves the session on success. |
| `resend`   | —                   | `[email]`                                                | Resend the verification code.                                                               |
| `whoami`   | —                   | —                                                        | Show the logged-in user, session expiry, API URL and keyring slot (local, unverified; never prints tokens). |
| `init`     | `i`                 | `[name]`, `--adopt`                                      | Create the remote project and local `.env` files. Defaults to the current directory's name; pass a name to override. `--adopt` records identity for an existing project. |
| `invite`   | —                   | `<email>`, `--role <writer\|reader>`                     | Invite a user to the current project (owner only; re-authenticates with your password). |
| `members`  | —                   | `role <email> <writer\|reader>`, `environment <email>`    | Change a member's role or re-pick their environment grants (owner-only; re-authenticates with your password). Listing members uses `list members`. |
| `invites`  | —                   | `list`, `cancel <id>`, `accept <token>`                   | List, cancel (owner, re-authenticates) or accept invitations. Accepting requires the invited account. |
| `add`      | `a`                 | `<environment>`                                          | Create an environment for the current project.                                               |
| `push`     | —                   | `[environment]`, `--show-values`                         | Push one environment, or all environments with local `.env.<env>` files when omitted. Values are hidden unless `--show-values`. |
| `pull`     | —                   | `[environment]`, `--show-values`, `--project-id <id>`     | Pull one environment, or all environments when omitted. Requires project membership. |
| `list`     | `ls`                | `projects`, `environments`, `members`                    | List your projects, the current project's environments, or its members (owner-only).        |
| `delete`   | `d`                 | `project [name]`, `environment <name>`, `-y`, `--yes`     | Delete a project (bare: current project; name deletes by name; password required) or an environment. |
| `help`     | `h`, `--help`, `-h` | —                                                        | Show help.                                                                                  |
