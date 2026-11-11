# Environment file format

Parsing is supported for:

```dotenv
# comments are preserved on pull
KEY=value
QUOTED="value with spaces"
SINGLE='literal #value'
export EXPORTED=value
INLINE=value # trailing comment
```

## Variable names and value escaping

Variable names must match the server's allowlist `^[A-Za-z_][A-Za-z0-9_]*$`
(max 256 bytes). Files with names outside this pattern are rejected on read,
and the same check is applied defensively on write.

Values that require quoting are written double-quoted with escapes. `$`, `` ` ``,
`!`, `"`, `\`, newlines, CR and TAB are escaped, so a value pulled from the
remote cannot execute command substitution or history expansion if the file is
sourced.
