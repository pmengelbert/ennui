#!/bin/bash

BRANCH_NAME="$(git rev-parse --abbrev-ref HEAD)"

if [[ "${BRANCH_NAME}" == "main" ]]; then
    echo "latest"
else
    echo "${BRANCH_NAME}"
fi

exit 0
