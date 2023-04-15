#!/bin/bash

TEMPLATE_FILE=$1

if [ ! -f "$TEMPLATE_FILE" ]; then
  echo "Error: Template file not found!" >&2
  exit 1
fi

while IFS= read -r line; do
  if [[ $line == *"{{"*"}}"* ]]; then
    file_to_include="${line//\{\{/}"
    file_to_include="${file_to_include//\}\}/}"
    if [ -f "$file_to_include" ]; then
      cat "$file_to_include"
    else
      echo "Error: File to include not found: $file_to_include" >&2
      exit 1
    fi
  else
    echo "$line"
  fi
done <"$TEMPLATE_FILE"

