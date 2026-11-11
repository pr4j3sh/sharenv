# Usage guide

## 1. Accounts and sessions

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
removes the local tokens.

## 2. Initialize a project

Run this from inside the project you want to track:

```bash
sharenv init                 # project name = current directory name
sharenv init my-project      # or choose one explicitly
```

This creates the remote project, the local `.sharenv/config.toml`, and one
`.env.<env>` file per default environment (`dev`, `prod`, `local`). It refuses
to reuse an existing remote project silently; use `--adopt` to opt in to that.

## 3. Push variables

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

## 4. Share the project or add an environment

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

The invitee accepts with (and receives the project id):

```bash
sharenv invites accept <token>
```

The invitee can then pull without a local `.sharenv` config:

```bash
sharenv pull --project-id <project_id> dev
```

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

## 5. Pull variables

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

## 6. List projects, environments, members

```bash
sharenv list projects       # ls p
sharenv list environments   # ls e
sharenv list members        # owner-only; members with roles and grants
```

`list environments` and `list members` use the project id from
`.sharenv/config.toml`; `list members` is owner-only.

## 7. Delete a project or environment

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
