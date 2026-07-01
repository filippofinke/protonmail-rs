#!/usr/bin/env bash
# Regenerate docs/CLI.md — the full command reference — from `protonmail-cli --help`.
# Run from the workspace root:  ./scripts/gen-cli-docs.sh
set -euo pipefail
cd "$(dirname "$0")/.."

cargo build -p protonmail-cli --quiet
BIN="$(pwd)/target/debug/protonmail-cli"
OUT="docs/CLI.md"
mkdir -p docs

leaf_top=(login logout whoami counts export watch index search sync)

# group -> its leaf subcommands
groups=(
  "messages:list search read send reply forward cancel-send spam ham unsubscribe empty label undelete receipt move trash delete mark star unstar"
  "conversations:list search read move trash mark star unstar snooze unsnooze"
  "attachments:list download"
  "drafts:list save edit delete"
  "filters:list create check delete enable disable"
  "contacts:list emails"
  "addresses:list update"
  "settings:get sign attach-public-key"
  "labels:list create delete update"
)

{
  printf '# protonmail-cli — full command reference\n\n'
  printf 'Complete help for every command and flag, generated from `protonmail-cli --help`\n'
  printf 'by `scripts/gen-cli-docs.sh`. For a quick overview see the [README](../README.md).\n\n'
  printf -- '- Every command also accepts the [global options](#protonmail-cli) (`--profile`, `--json`, `-v/-vv/-vvv`, `--client`, …); clap repeats them in each command'"'"'s help.\n'
  printf -- '- `<REF>` = an exact message id **or** free text that uniquely matches one message.\n\n'

  printf '## protonmail-cli\n\n```text\n'; "$BIN" --help 2>&1; printf '```\n\n'

  for c in "${leaf_top[@]}"; do
    printf '## protonmail-cli %s\n\n```text\n' "$c"; "$BIN" "$c" --help 2>&1; printf '```\n\n'
  done

  for entry in "${groups[@]}"; do
    g="${entry%%:*}"; leaves="${entry#*:}"
    printf '## protonmail-cli %s\n\n```text\n' "$g"; "$BIN" "$g" --help 2>&1; printf '```\n\n'
    for leaf in $leaves; do
      printf '### protonmail-cli %s %s\n\n```text\n' "$g" "$leaf"; "$BIN" "$g" "$leaf" --help 2>&1; printf '```\n\n'
    done
  done
} > "$OUT"

echo "wrote $OUT ($(wc -l < "$OUT") lines)"
