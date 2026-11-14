# Usage guide

## Accounts and sessions

```bash
sharenv register   # r
sharenv login      # l
sharenv logout
```

If the server has email verification enabled, `register` emails you a one-time
code and asks for it before logging you in. Verify later (or on another
machine) with `sharenv verify [email]`; `sharenv resend` sends a new code.
Sessions use short-lived access tokens with a rotating refresh token; the CLI
refreshes automatically. `sharenv logout` revokes the session server-side and
removes the local tokens. `sharenv whoami` shows the logged-in user, session
expiry and which keyring slot is in use. Add `--json` for machine-readable
output (`list`, `whoami`, `push`, `pull`); progress goes to stderr.

Keyring slots are derived per server (`<token_account>-<fnv1a64(base_url)>`) so
dev and production sessions don't overwrite each other; pass `--profile <name>`
on any command to use a named slot instead.

## Initialize a project

Run this from inside the project you want to track:

```bash
sharenv init                 # project name = current directory name
sharenv init my-project      # or choose one explicitly
```

This creates the remote project, the local `.sharenv/config.toml`, and one
`.env.<env>` file per default environment (`dev`, `prod`, `local`).

Init checks your local files _before_ touching anything remote: if an existing
`.env.<env>` is not a safe secret file (wrong permissions, a symlink, not owned
by you), it stops with the `chmod 600` fix and creates no project, remote or
otherwise. If a previous run left a partial setup behind (a remote project or
environment exists but a local `.env.<env>` file does not), running `init`
again repairs the missing pieces; nothing is created twice. When a local
project id already matches a remote project it reports "already initialized".
Use `--adopt` to record identity metadata for an existing, initialized project
without creating anything.

## Push variables

Push one environment:

```bash
sharenv push dev
```

Or push every remote environment that has a matching `.env.<env>` file:

```bash
sharenv push
```

New variables are created. If a variable already exists remotely with a
different value, `push` asks whether to update the remote one. Values are
hidden by default; pass `--show-values` to print sanitized values.

`push` shows the destination project and owner first. Pushing to a project you
do not own always asks for confirmation.

## Share the project or add an environment

Only the project owner can share it. Sharing uses invitations: the teammate
must accept before they gain access. Run this from the initialized project
directory:

```bash
sharenv invite teammate@example.com
```

You'll be asked for the role (`writer` or `reader`, default `reader`), which
environments to grant, and your password (the API re-authenticates you before
creating the invitation). If the server has no mail provider configured, the
invitation token is printed once — send it to your teammate out-of-band.

The invitee accepts with:

```bash
sharenv invites accept <token>
```

Acceptance links the current directory: the CLI writes `.sharenv/config.toml`
with the project id and its identity metadata (file mode `0600`), so the
invitee can run `pull`, `push`, `add` and `list environments` there directly:

```bash
cd the-project
sharenv pull dev
```

The link is never written over an existing binding — if the directory already
records a project (or contains an unreadable config), it is left untouched and
the accept hint tells them to use `sharenv pull --project-id <project_id> dev`
instead. `pull` also works without a binding via `--project-id`; `push`
requires a linked directory.

Manage pending invitations for the current project:

```bash
sharenv invites list
sharenv invites cancel <invitation_id>
```

Once they've accepted, list members and manage roles and grants (owner-only;
the API re-authenticates you with your password):

```bash
sharenv list members
sharenv members role teammate@example.com writer
sharenv members environment teammate@example.com
```

Create another environment for the current project:

```bash
sharenv add staging
sharenv add staging    # again: no-op "already exists"
```

This creates the environment remotely and adds an empty `.env.staging` file if
it does not exist yet.

## Pull variables

```bash
sharenv pull dev
```

Run `sharenv pull` with no environment to pull all remote environments. By
default `pull` reads the project id from `.sharenv/config.toml`. Pass
`--project-id <project_id>` to pull a shared project from any directory; with
no environment that pulls all of its environments too.

You must be a member of the project to pull it. The CLI resolves the project
from the server first and refuses non-members before fetching any variables.
Values are hidden by default; pass `--show-values` to print them. Existing
comments and formatting in the local file are preserved; the values are hidden
no matter which you pick.

## List projects, environments, members

```bash
sharenv list projects       # ls p
sharenv list environments   # ls e
sharenv list members        # owner-only; members with roles and grants
```

`list environments` and `list members` use the project id from
`.sharenv/config.toml`; `list members` is owner-only.

## Delete a project or environment

Run this from the initialized project directory. All commands ask for
confirmation; add `--yes` (or `-y`) to skip the prompt.

```bash
sharenv delete project                   # current project (id from .sharenv/config.toml)
sharenv delete project my-project        # project by name
sharenv delete environment dev           # one environment
```

`delete project` with no name deletes the current project and removes that
local config on success. A name deletes by name through the API; the local
config is removed only if the deleted project id matches. Deleting a project
requires your password, even with `--yes`; local `.env.<env>` files are left
untouched.
