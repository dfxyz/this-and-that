#!/usr/bin/bash

wd=$(dirname "$0")
cd "$wd" || exit 1

volume_name=$1
if [[ -z "$volume_name" ]]; then
    echo "volume name not specified"
    exit 1
fi
archive_filename="$volume_name.tar.gz"
if [[ ! -f "$archive_filename" ]]; then
    echo "archive file '$archive_filename' not found"
    exit 1
fi

echo "restoring volume '$volume_name'..."

cp ./restore.yaml.template ./restore.yaml
sed -i -e "s/VOLUME_NAME/$volume_name/g" ./restore.yaml

docker compose -f ./restore.yaml up && docker compose -f ./restore.yaml down
rm ./restore.yaml
