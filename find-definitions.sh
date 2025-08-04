#!/bin/bash
# find-definitions.sh - Find all Rust definitions in file order

set -e

echo "=== Rust Code Definitions (by file order) ==="
echo

# Single rg call to find all definitions, sorted by file and line number
rg --no-heading --line-number --color=never \
   '^[[:space:]]*(pub[[:space:]]+)?(struct|enum|trait|impl|fn|mod)[[:space:]]+\w+' \
   src/ \
   | sort -t: -k1,1 -k2,2n \
   | while IFS=: read -r file line content; do
     # Extract the definition type and name for cleaner display
     type=$(echo "$content" | sed -E 's/^[[:space:]]*(pub[[:space:]]+)?([a-z_]+)[[:space:]]+.*/\2/')
     case "$type" in
       "struct") icon="🏗️" ;;
       "enum") icon="🔢" ;;
       "trait") icon="🎯" ;;
       "impl") icon="🧩" ;;
       "fn") icon="⚙️" ;;
       "mod") icon="📦" ;;
       *) icon="❓" ;;
     esac

     printf "%-30s %3s %s %s\n" "$file" "$line" "$icon" "$content"
   done
