#!/bin/bash -x

docker exec -ti vault vault kv put -mount=secret guestowner1/workload-id/secret password=test
docker exec -ti vault vault kv get -mount=secret guestowner1/workload-id/secret

docker exec -ti vault sh -c 'tee readonly.hcl <<EOF
 path "secret/data/guestowner1/workload-id/*" {
   capabilities = ["read"]
 }
EOF
'
docker exec -ti vault vault policy write kbs readonly.hcl

docker exec -ti vault vault token create -policy="kbs" -field=token > token-vault-kb
