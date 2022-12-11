#!/usr/bin/bash

wd=$(dirname "$0")
cd "$wd" || exit 1

volume_name=$1
if [[ -z "$volume_name" ]]; then
    echo "volume name not specified"
    exit 1
fi

echo "backing up volume '$volume_name'..."

cp ./backup.yaml.template ./backup.yaml
sed -i -e "s/VOLUME_NAME/$volume_name/g" ./backup.yaml

docker compose -f ./backup.yaml up && docker compose -f ./backup.yaml down
rm ./backup.yaml
