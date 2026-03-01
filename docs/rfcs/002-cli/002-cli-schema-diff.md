---
name: schema-diff
title: Schema diff output
status: Draft
date: 2026-03-01
---

## Problem

`gfs diff` needs to serve two audiences with conflicting needs: AI agents (structured, parseable) and humans (visual, annotated). A single format serves neither well.

---

## Proposal

Two modes, agentic by default:

```sh
gfs diff <from> <to>           # default: agentic format
gfs diff <from> <to> --pretty  # human-readable format
```

### Agentic Format (default)

Line-oriented, one mutation per line, key=value attributes.

```shell
GFS_DIFF v1 from=a3f1c2 to=b7d4e9 breaking=true
TABLE DROP orders_archive breaking=true
COLUMN DROP users.legacy_id type=int breaking=true
COLUMN ADD users.verified_at type=timestamp nullable=true
COLUMN MODIFY users.email type=varchar(100)->varchar(255)
FK ADD products.fk_category from=category_id to=categories(id)
```

**Rules:**

- First line is always the header with format version
- Format: `ENTITY OPERATION target key=value ...`
- Deterministic output (stable sort by entity type, then name)
- No color, no decoration

### Pretty Format (`--pretty`)

```shell
Schema diff  main@a3f1c2 → main@b7d4e9
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  ✕ dropped   orders_archive
  MODIFIED    users
  │  ✕ column   legacy_id     int          [BREAKING]
  │  + column   verified_at   timestamp    nullable
  │  ~ column   email         varchar(100) → varchar(255)
  MODIFIED    products
  │  + fk       fk_category   category_id → categories(id)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Summary   1 modified · 1 dropped
  Risk      ⚠ BREAKING CHANGES
```

---

## Exit Codes

| Code | Meaning |
|------|---------|
| 0    | No changes |
| 1    | Safe changes only |
| 2    | Breaking changes present |

---

## Legend

Applies to both formats.

| Symbol | Line prefix | Meaning |
|--------|-------------|---------|
| `+`    | `ADD`       | Entity was added |
| `✕`    | `DROP`      | Entity was dropped |
| `~`    | `MODIFY`    | Entity was modified |

---

## Breaking Change Rules

| Operation | Breaking |
|-----------|----------|
| DROP TABLE / COLUMN | Yes |
| RENAME COLUMN | Yes |
| Type narrowing | Yes |
| NOT NULL on existing column | Warning |
| ADD COLUMN nullable | No |
| Type widening | No |

---

## Colors (Pretty Format)

Applied to the `--pretty` format only. No color is ever emitted in the default agentic format.

| Color  | Applies to |
|--------|------------|
| Red    | `✕` DROP operations, `[BREAKING]` tag |
| Green  | `+` ADD operations |
| Yellow | `~` MODIFY operations, `⚠` risk line |
| Dim    | `│` tree connectors, column type details |
| Bold   | Table names, summary line |

**Example (annotated):**
```
  ✕ dropped   orders_archive          ← red
  MODIFIED    users                   ← bold
  │  ✕ column   legacy_id   [BREAKING]  ← red
  │  + column   verified_at  nullable    ← green
  │  ~ column   email   varchar(100) → varchar(255)  ← yellow
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Risk      ⚠ BREAKING CHANGES        ← yellow ⚠, red text
```

**Rules:**
- Use ANSI escape codes
- Disable automatically when stdout is not a TTY (piped or redirected)
- Respect `NO_COLOR` environment variable (https://no-color.org)
- `--no-color` flag forces disable

## Out of Scope

- **JSON output** — future RFC
- **Diff of data** — not schema
- **Migration script generation**